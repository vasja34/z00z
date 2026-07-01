use crate::backend::CryptoBackend;
use crate::backend::{CommitFactory, RangeProofSvc, TariCryptoBackend};
use crate::{
    default_backend, CryptoError, KernelSignature, RangeProof, Z00ZCommitment, Z00ZRistrettoPoint,
    Z00ZScalar, AGGREGATION_FACTOR, MIN_VALUE_PROMISE, RANGE_PROOF_BITS,
};

/// Initialize cryptographic backend at application startup.
///
/// Forces eager initialization of all lazy-loaded crypto services
/// (Bulletproof+ service, commitment factory) to detect failures early.
/// Safe to call multiple times (idempotent - subsequent calls are no-ops).
///
/// # Panics
///
/// Panics immediately if cryptographic services fail to initialize.
/// This fail-fast behavior prevents operating in a broken crypto state.
pub fn initialize() {
    TariCryptoBackend::initialize()
}

/// Create a Pedersen commitment hiding an amount.
pub fn create_commitment(
    amount: u64,
    blinding: &Z00ZScalar,
) -> Result<Z00ZCommitment, CryptoError> {
    if blinding.is_zero() {
        return Err(CryptoError::InvalidBlindingFactorZero);
    }

    Ok(default_backend().create_commitment(amount, blinding))
}

/// Generate a zero-knowledge range proof for an amount.
pub fn create_range_proof(
    amount: u64,
    blinding: &Z00ZScalar,
    bits: usize,
    minimum_value_promise: u64,
) -> Result<RangeProof, CryptoError> {
    if blinding.is_zero() {
        return Err(CryptoError::InvalidBlindingFactorZero);
    }

    default_backend().create_range_proof(amount, blinding, bits, minimum_value_promise)
}

/// Generate a deterministic V1 range proof using caller-provided RNG.
///
/// This helper is intended for replayable test and simulation flows that need
/// byte-stable proofs under a fixed seed.
pub fn create_range_proof_rng<R: rand::CryptoRng + rand::RngCore>(
    amount: u64,
    blinding: &Z00ZScalar,
    bits: usize,
    minimum_value_promise: u64,
    rng: &mut R,
) -> Result<RangeProof, CryptoError> {
    if blinding.is_zero() {
        return Err(CryptoError::InvalidBlindingFactorZero);
    }
    if !(1..=64).contains(&bits) || bits != RANGE_PROOF_BITS {
        return Err(CryptoError::InvalidParameters { param: "bits" });
    }
    if minimum_value_promise != MIN_VALUE_PROMISE || amount < minimum_value_promise {
        return Err(CryptoError::InvalidParameters {
            param: "minimum_value_promise",
        });
    }

    let key = blinding.reveal().clone();
    let svc = RangeProofSvc::init(
        RANGE_PROOF_BITS,
        AGGREGATION_FACTOR,
        CommitFactory::default(),
    )
    .map_err(|_| CryptoError::ServiceInitializationFailed)?;

    svc.construct_proof_with_rng(&key, amount, rng)
        .map_err(|_| CryptoError::ProofGenerationFailed)
}

pub trait VerifyCommitmentInput {
    fn commitment_ref(&self) -> Option<&Z00ZCommitment>;
}

impl<T> VerifyCommitmentInput for &T
where
    T: VerifyCommitmentInput + ?Sized,
{
    fn commitment_ref(&self) -> Option<&Z00ZCommitment> {
        (*self).commitment_ref()
    }
}

impl VerifyCommitmentInput for Z00ZCommitment {
    fn commitment_ref(&self) -> Option<&Z00ZCommitment> {
        Some(self)
    }
}

impl VerifyCommitmentInput for Result<Z00ZCommitment, CryptoError> {
    fn commitment_ref(&self) -> Option<&Z00ZCommitment> {
        self.as_ref().ok()
    }
}

/// Verify a zero-knowledge range proof against a commitment.
pub fn verify_range_proof<C>(
    proof: &RangeProof,
    commitment: &C,
    bits: usize,
    aggregation_factor: usize,
    minimum_value_promise: u64,
) -> Result<(), CryptoError>
where
    C: VerifyCommitmentInput + ?Sized,
{
    let commitment = commitment
        .commitment_ref()
        .ok_or(CryptoError::InvalidCommitment)?;
    default_backend().verify_range_proof(
        proof,
        commitment,
        bits,
        aggregation_factor,
        minimum_value_promise,
    )
}

/// Sign a message with a Schnorr signature using a secret scalar.
///
/// Available only on non-WASM builds.
#[cfg(not(target_arch = "wasm32"))]
pub fn sign_kernel_signature(
    secret: &Z00ZScalar,
    message: impl AsRef<[u8]>,
    rng: &mut (impl rand::RngCore + rand::CryptoRng),
) -> Result<KernelSignature, CryptoError> {
    KernelSignature::sign(secret.inner(), message, rng).map_err(|_| CryptoError::BackendError {
        context: "schnorr_sign",
    })
}

/// Verify a Schnorr signature against a public key wrapper.
pub fn verify_kernel_signature(
    signature: &KernelSignature,
    public_key: &Z00ZRistrettoPoint,
    message: impl AsRef<[u8]>,
) -> bool {
    signature.verify(public_key.inner(), message)
}

/// Derive a deterministic hash with domain separation.
pub fn derive_hash(domain: &[u8], data: &[&[u8]]) -> [u8; 32] {
    default_backend().derive_hash(domain, data)
}

/// Batch verify multiple range proofs simultaneously.
pub fn batch_verify_range_proofs(
    proofs: &[&RangeProof],
    commitments: &[&Z00ZCommitment],
    bits: usize,
    aggregation_factor: usize,
    minimum_value_promises: &[u64],
) -> Result<(), CryptoError> {
    default_backend().batch_verify_range_proofs(
        proofs,
        commitments,
        bits,
        aggregation_factor,
        minimum_value_promises,
    )
}

/// Batch verify range proofs with optional timeout.
pub fn batch_verify_range_proofs_with(
    proofs: &[&RangeProof],
    commitments: &[&Z00ZCommitment],
    bits: usize,
    aggregation_factor: usize,
    minimum_value_promises: &[u64],
    timeout: Option<std::time::Duration>,
) -> Result<(), CryptoError> {
    let start = std::time::Instant::now();
    let result = batch_verify_range_proofs(
        proofs,
        commitments,
        bits,
        aggregation_factor,
        minimum_value_promises,
    );

    if let Some(timeout) = timeout {
        if start.elapsed() > timeout {
            return Err(CryptoError::BatchTimeout);
        }
    }

    result
}
