/// Aggregated stealth receiver keys derived from one receiver secret.
pub struct ReceiverKeys {
    receiver_secret: Hidden<ReceiverSecret>,
    /// Public receiver handle used for routing.
    pub owner_handle: [u8; 32],
    view_version: u32,
    prior_view_sks: Vec<Hidden<Z00ZScalar>>,
    view_sk: Hidden<Z00ZScalar>,
    /// Public view key.
    pub view_pk: Z00ZRistrettoPoint,
    identity_sk: Hidden<Z00ZScalar>,
    /// Public identity key.
    pub identity_pk: Z00ZRistrettoPoint,
}

impl ReceiverKeys {
    /// Derives the full receiver key bundle from a master receiver secret.
    pub fn from_receiver_secret(receiver_secret: ReceiverSecret) -> Result<Self, StealthKeyError> {
        receiver_secret.validate_usable()?;
        let owner_handle = derive_owner_handle(&receiver_secret);
        let view_sk = derive_view_secret_key(&receiver_secret)?;
        let view_pk = derive_view_public_key(&view_sk)?;
        let identity_sk = derive_identity_secret_key(&receiver_secret, 0)?;
        let identity_pk = derive_identity_public_key(&identity_sk)?;

        Ok(Self {
            receiver_secret: receiver_secret.into_hidden(),
            owner_handle,
            view_version: 0,
            prior_view_sks: Vec::new(),
            view_sk: Hidden::hide(view_sk),
            view_pk,
            identity_sk: Hidden::hide(identity_sk),
            identity_pk,
        })
    }

    /// Derives a path-scoped receiver bundle for BIP-44 partitioning.
    pub fn from_receiver_secret_with_path(
        receiver_secret: &ReceiverSecret,
        path: &Bip44Path,
    ) -> Result<Self, StealthKeyError> {
        let scoped = derive_path_secret(receiver_secret, path)?;
        Self::from_receiver_secret(scoped)
    }

    /// Returns the underlying receiver secret for internal wallet flows.
    pub(crate) fn reveal_receiver_secret(&self) -> &ReceiverSecret {
        self.receiver_secret.reveal()
    }

    /// Returns the active view secret key.
    pub fn reveal_view_sk(&self) -> &Z00ZScalar {
        self.view_sk.reveal()
    }

    /// Returns the active identity secret key.
    pub fn reveal_identity_sk(&self) -> &Z00ZScalar {
        self.identity_sk.reveal()
    }

    /// Returns the current and prior view secret keys in newest-first order.
    pub fn all_view_sks(&self) -> Vec<Z00ZScalar> {
        let mut out = Vec::with_capacity(self.prior_view_sks.len().saturating_add(1));
        out.push(self.reveal_view_sk().dangerous_clone());
        for prior in &self.prior_view_sks {
            out.push(prior.reveal().dangerous_clone());
        }
        out
    }

    /// Rotates the active view key and returns an updated receiver card.
    pub fn rotate_view(&mut self) -> Result<ReceiverCard, StealthKeyError> {
        let next_version = self
            .view_version
            .checked_add(1)
            .ok_or(StealthKeyError::InvalidSecretKey)?;
        let next_view_sk =
            derive_rotated_view_secret_key(self.reveal_receiver_secret(), next_version)?;
        let next_view_pk = derive_view_public_key(&next_view_sk)?;

        self.prior_view_sks
            .push(Hidden::hide(self.reveal_view_sk().dangerous_clone()));

        self.view_version = next_version;
        self.view_sk = Hidden::hide(next_view_sk);
        self.view_pk = next_view_pk;

        self.export_receiver_card()
    }

    /// Exports a signed receiver card for distribution.
    pub fn export_receiver_card_with_rng<R>(&self, rng: &mut R) -> Result<ReceiverCard, StealthKeyError>
    where
        R: rand::CryptoRng + rand::RngCore,
    {
        let view_pk = pk_bytes(&self.view_pk)?;
        let identity_pk = pk_bytes(&self.identity_pk)?;

        let mut card = ReceiverCard {
            version: 1,
            owner_handle: self.owner_handle,
            view_pk,
            identity_pk,
            card_id: None,
            metadata: None,
            signature: [0u8; 64],
        };

        card.sign_with_rng(self.reveal_identity_sk(), rng)?;
        Ok(card)
    }

    /// Exports a signed receiver card for distribution.
    pub fn export_receiver_card(&self) -> Result<ReceiverCard, StealthKeyError> {
        let mut rng = SystemRngProvider.rng();
        let card = self.export_receiver_card_with_rng(&mut rng)?;
        Ok(card)
    }
}

#[cfg(feature = "test-params-fast")]
/// Micro-benchmarks receiver bundle derivation for fast-test environments.
pub fn benchmark_recv_keys(iterations: usize) -> Result<Duration, StealthKeyError> {
    if iterations == 0 {
        return Ok(Duration::ZERO);
    }

    let start = Instant::now();
    for index in 0..iterations {
        let mut bytes = [0x11u8; 32];
        bytes[0] = (index as u8).wrapping_add(1);
        let secret = ReceiverSecret::from_bytes(std::hint::black_box(bytes))?;
        let keys = ReceiverKeys::from_receiver_secret(secret)?;
        std::hint::black_box(keys.owner_handle);
    }
    Ok(start.elapsed())
}

fn derive_path_secret(
    receiver_secret: &ReceiverSecret,
    path: &Bip44Path,
) -> Result<ReceiverSecret, StealthKeyError> {
    let path_bytes = path.to_bytes();
    let scoped = hash_zk::<WalletBIP44Domain>(
        "RECV_PATH",
        &[receiver_secret.as_bytes(), path_bytes.as_slice()],
    );
    ReceiverSecret::from_raw(scoped)
}

fn pk_bytes(public_key: &Z00ZRistrettoPoint) -> Result<[u8; 32], StealthKeyError> {
    let bytes = public_key.as_bytes();
    if bytes.len() != 32 {
        return Err(StealthKeyError::PublicKeyEncodingFailed);
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(bytes);
    Ok(out)
}

fn derive_storage_key(password: &[u8], salt: &[u8; SALT_LEN]) -> Result<[u8; 32], StealthKeyError> {
    if password.is_empty() {
        return Err(StealthKeyError::EmptyPassword);
    }

    let key = derive_argon2id32_key(password, salt, &kdf_params())
        .map_err(|_| StealthKeyError::KeyDeriveFailed)?;
    Ok(*key.reveal())
}

#[cfg(any(test, feature = "test-params-fast"))]
fn kdf_params() -> Argon2Params {
    Argon2Params {
        memory: 16,
        iterations: 1,
        parallelism: 1,
    }
}

#[cfg(not(any(test, feature = "test-params-fast")))]
fn kdf_params() -> Argon2Params {
    Argon2Params::moderate()
}

fn identity_payload(message: &[u8], context: &[u8]) -> Vec<u8> {
    let domain = hash_zk::<IdentitySignatureDomain>("IDENTITY_SIG", &[context, message]);
    let mut payload = frame_bytes(IdentitySignatureDomain::domain().as_bytes());
    payload.extend_from_slice(&frame_bytes(context));
    payload.extend_from_slice(&frame_bytes(message));
    payload.extend_from_slice(&frame_bytes(&domain));
    payload
}
