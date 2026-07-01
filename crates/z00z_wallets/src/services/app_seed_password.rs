impl AppService {
    fn generate_seed_phrase_24(&self) -> WalletResult<String> {
        let mut rng = self.core_app.rng_provider.rng();
        generate_seed_phrase_24_english(|dest| rng.fill_bytes_ext(dest))
            .map(|hidden| hidden.with_revealed(|s| s.to_string()))
    }

    fn verify_password_confirmation(password: &SafePassword) -> WalletResult<()> {
        let password_bytes = password.reveal().as_slice();
        if password_bytes.is_empty() {
            return Err(WalletError::InvalidParams("Password required".to_string()));
        }
        Ok(())
    }

    fn validate_seed_phrase_24_english(seed_phrase: &str) -> WalletResult<()> {
        let phrase = seed_phrase.trim();
        if phrase.is_empty() {
            return Err(WalletError::InvalidParams(
                "Seed phrase is required".to_string(),
            ));
        }

        let seed_phrase = match SeedPhrase24::parse_in(MnemonicLanguage::English, phrase) {
            Ok(v) => v,
            Err(crate::key::SeedPhraseError::InvalidWordCount { .. }) => {
                return Err(WalletError::InvalidParams(
                    "Seed phrase must contain 24 words".to_string(),
                ));
            }
            Err(_) => {
                return Err(WalletError::InvalidParams(
                    "Invalid seed phrase".to_string(),
                ));
            }
        };

        let mut entropy = seed_phrase
            .to_bip39_entropy_bytes()
            .map_err(|_| WalletError::InvalidParams("Invalid seed phrase".to_string()))?;

        if entropy.len() != 32 {
            entropy.fill(0);
            return Err(WalletError::InvalidParams(
                "Seed phrase must produce 32 bytes of entropy".to_string(),
            ));
        }

        entropy.fill(0);

        Ok(())
    }

    async fn verify_unlocked(&self, wallet_id: &PersistWalletId) -> WalletResult<()> {
        self.wallets.check_auto_lock().await?;
        let state = self.wallets.get_wallet_state(wallet_id).await?;
        if !state.is_unlocked() {
            return Err(WalletError::Locked);
        }
        Ok(())
    }

    async fn update_activity(&self, wallet_id: &PersistWalletId) {
        let _ = self.wallets.update_activity(wallet_id).await;
    }
}