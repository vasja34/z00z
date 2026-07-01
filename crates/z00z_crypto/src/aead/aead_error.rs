use thiserror::Error;

#[derive(Debug, Error)]
pub enum AeadError {
    #[error("AEAD operation failed")]
    Crypto,

    #[error("Random number generation failed")]
    Random,
}
