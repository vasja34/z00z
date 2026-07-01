//! In-memory wallet session manager (Phase 1).
//!
//! Owns the only key-bearing session objects and validates session tokens.

#![cfg(not(target_arch = "wasm32"))]

use crate::rpc::types::{common::PersistWalletId, wallet::SessionToken};
use crate::{WalletError, WalletResult};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use subtle::ConstantTimeEq;
use tokio::sync::RwLock;
use z00z_utils::time::TimeProvider;

#[derive(Debug)]
pub(crate) struct SessionHandle {
    inner: Arc<SessionInner>,
}

impl SessionHandle {
    fn new(wlt_session: crate::db::WalletSession) -> Self {
        Self {
            inner: Arc::new(SessionInner {
                wlt_session: Mutex::new(Some(wlt_session)),
            }),
        }
    }

    pub(crate) fn with_wallet_session<F, R>(&self, f: F) -> WalletResult<R>
    where
        F: FnOnce(&crate::db::WalletSession) -> WalletResult<R>,
    {
        let guard = self
            .inner
            .wlt_session
            .lock()
            .map_err(|_| WalletError::SessionInvalid)?;

        let Some(session) = guard.as_ref() else {
            return Err(WalletError::SessionInvalid);
        };

        f(session)
    }

    pub(crate) fn take_wallet_session_mut<F, R>(&self, f: F) -> WalletResult<R>
    where
        F: FnOnce(&mut crate::db::WalletSession) -> WalletResult<R>,
    {
        let mut guard = self
            .inner
            .wlt_session
            .lock()
            .map_err(|_| WalletError::SessionInvalid)?;

        let Some(session) = guard.as_mut() else {
            return Err(WalletError::SessionInvalid);
        };

        let outcome = f(session)?;

        let removed = guard.take();
        drop(removed);

        Ok(outcome)
    }

    fn revoke_and_drop(&self) {
        if let Ok(mut guard) = self.inner.wlt_session.lock() {
            let removed = guard.take();
            drop(removed);
        }
    }
}

#[derive(Debug)]
struct SessionInner {
    wlt_session: Mutex<Option<crate::db::WalletSession>>,
}

fn ct_eq_token(left: &str, right: &str) -> bool {
    left.as_bytes().ct_eq(right.as_bytes()).unwrap_u8() == 1
}

#[derive(Debug)]
struct ManagedSession {
    token_hex: String,
    created_at_ms: u64,
    expires_at_ms: u64,
    last_activity_at_ms: u64,
    session_handle: SessionHandle,
}

/// Process-local session manager.
///
/// Phase 1 contract:
/// - Exactly one owner of key-bearing state exists: the session manager.
/// - All sensitive operations must validate a `SessionToken` and use the cached session.
pub(crate) struct WalletSessionManager {
    time_provider: Arc<dyn TimeProvider>,
    sessions: RwLock<BTreeMap<PersistWalletId, ManagedSession>>,
}

impl std::fmt::Debug for WalletSessionManager {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WalletSessionManager")
            .finish_non_exhaustive()
    }
}

impl WalletSessionManager {
    pub(crate) fn new(time_provider: Arc<dyn TimeProvider>) -> Self {
        Self {
            time_provider,
            sessions: RwLock::new(BTreeMap::new()),
        }
    }

    fn current_time_ms(&self) -> WalletResult<u64> {
        self.time_provider
            .try_unix_timestamp_ms()
            .map_err(|error| WalletError::InvalidConfig(format!("clock unavailable: {error}")))
    }

    fn revoke_wallet_session_locked(
        sessions: &mut BTreeMap<PersistWalletId, ManagedSession>,
        wallet_id: &PersistWalletId,
    ) {
        let removed = sessions.remove(wallet_id);
        if let Some(removed) = &removed {
            removed.session_handle.revoke_and_drop();
        }
    }

    pub(crate) async fn drop_session(&self, wallet_id: &PersistWalletId) {
        let removed = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(wallet_id)
        };

        if let Some(session) = &removed {
            session.session_handle.revoke_and_drop();
        }

        drop(removed);
    }

    pub(crate) async fn touch(&self, wallet_id: &PersistWalletId, now_ms: u64, timeout_ms: u64) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(wallet_id) {
            session.last_activity_at_ms = now_ms;
            session.expires_at_ms = now_ms.saturating_add(timeout_ms);
        }
    }

    pub(crate) async fn existing_token(
        &self,
        wallet_id: &PersistWalletId,
        _now_ms: u64,
        timeout_ms: u64,
    ) -> Option<SessionToken> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(wallet_id)?;
        let now_ms = self.current_time_ms().ok()?;

        if now_ms >= session.expires_at_ms {
            Self::revoke_wallet_session_locked(&mut sessions, wallet_id);
            return None;
        }

        session.last_activity_at_ms = now_ms;
        session.expires_at_ms = now_ms.saturating_add(timeout_ms);

        Some(SessionToken {
            token: session.token_hex.clone(),
            wallet_id: wallet_id.clone(),
            created_at: session.created_at_ms,
            expires_at: session.expires_at_ms,
            last_activity_at: now_ms,
            permissions: vec![],
        })
    }

    pub(crate) async fn insert(
        &self,
        wallet_id: &PersistWalletId,
        token_hex: String,
        now_ms: u64,
        timeout_ms: u64,
        wlt_session: crate::db::WalletSession,
    ) -> SessionToken {
        let expires_at_ms = now_ms.saturating_add(timeout_ms);

        let session_token = SessionToken {
            token: token_hex.clone(),
            wallet_id: wallet_id.clone(),
            created_at: now_ms,
            expires_at: expires_at_ms,
            last_activity_at: now_ms,
            permissions: vec![],
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(
            wallet_id.clone(),
            ManagedSession {
                token_hex,
                created_at_ms: now_ms,
                expires_at_ms,
                last_activity_at_ms: now_ms,
                session_handle: SessionHandle::new(wlt_session),
            },
        );

        session_token
    }

    pub(crate) async fn verify(
        &self,
        token: &SessionToken,
        _now_ms: u64,
        timeout_ms: u64,
    ) -> WalletResult<SessionHandle> {
        if token.token.trim().is_empty() {
            return Err(WalletError::SessionInvalid);
        }

        let mut sessions = self.sessions.write().await;
        let Some(session) = sessions.get_mut(&token.wallet_id) else {
            return Err(WalletError::SessionInvalid);
        };
        let now_ms = self.current_time_ms()?;

        if !ct_eq_token(&session.token_hex, &token.token) {
            return Err(WalletError::SessionInvalid);
        }

        if now_ms >= session.expires_at_ms {
            Self::revoke_wallet_session_locked(&mut sessions, &token.wallet_id);
            return Err(WalletError::SessionExpired);
        }

        session.last_activity_at_ms = now_ms;
        session.expires_at_ms = now_ms.saturating_add(timeout_ms);

        Ok(SessionHandle {
            inner: Arc::clone(&session.session_handle.inner),
        })
    }

    pub(crate) async fn validate(&self, token: &SessionToken, _now_ms: u64) -> WalletResult<()> {
        if token.token.trim().is_empty() {
            return Err(WalletError::SessionInvalid);
        }

        let mut sessions = self.sessions.write().await;
        let Some(session) = sessions.get_mut(&token.wallet_id) else {
            return Err(WalletError::SessionInvalid);
        };
        let now_ms = self.current_time_ms()?;

        if !ct_eq_token(&session.token_hex, &token.token) {
            return Err(WalletError::SessionInvalid);
        }

        if now_ms >= session.expires_at_ms {
            Self::revoke_wallet_session_locked(&mut sessions, &token.wallet_id);
            return Err(WalletError::SessionExpired);
        }

        Ok(())
    }

    pub(crate) async fn get_session_handle_without_touch(
        &self,
        token: &SessionToken,
        _now_ms: u64,
    ) -> WalletResult<SessionHandle> {
        if token.token.trim().is_empty() {
            return Err(WalletError::SessionInvalid);
        }

        let mut sessions = self.sessions.write().await;
        let Some(session) = sessions.get_mut(&token.wallet_id) else {
            return Err(WalletError::SessionInvalid);
        };
        let now_ms = self.current_time_ms()?;

        if !ct_eq_token(&session.token_hex, &token.token) {
            return Err(WalletError::SessionInvalid);
        }

        if now_ms >= session.expires_at_ms {
            Self::revoke_wallet_session_locked(&mut sessions, &token.wallet_id);
            return Err(WalletError::SessionExpired);
        }

        Ok(SessionHandle {
            inner: Arc::clone(&session.session_handle.inner),
        })
    }

    pub(crate) async fn is_wallet_session_active(
        &self,
        wallet_id: &PersistWalletId,
        _now_ms: u64,
    ) -> bool {
        let mut sessions = self.sessions.write().await;
        let Some(session) = sessions.get_mut(wallet_id) else {
            return false;
        };
        let Ok(now_ms) = self.current_time_ms() else {
            return false;
        };

        if now_ms >= session.expires_at_ms {
            Self::revoke_wallet_session_locked(&mut sessions, wallet_id);
            return false;
        }

        true
    }

    pub(crate) async fn session_for_wallet(
        &self,
        wallet_id: &PersistWalletId,
        _now_ms: u64,
        timeout_ms: u64,
    ) -> WalletResult<SessionHandle> {
        let mut sessions = self.sessions.write().await;
        let Some(session) = sessions.get_mut(wallet_id) else {
            return Err(WalletError::SessionInvalid);
        };
        let now_ms = self.current_time_ms()?;

        if now_ms >= session.expires_at_ms {
            Self::revoke_wallet_session_locked(&mut sessions, wallet_id);
            return Err(WalletError::SessionExpired);
        }

        session.last_activity_at_ms = now_ms;
        session.expires_at_ms = now_ms.saturating_add(timeout_ms);

        Ok(SessionHandle {
            inner: Arc::clone(&session.session_handle.inner),
        })
    }
}
