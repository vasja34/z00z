use crate::CryptoError;

use super::{MAX_AAD_SIZE, MAX_AAD_SIZE_EXTENDED};

pub fn build_aad(domain: &str, context: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + domain.len() + 1 + context.len());
    out.extend_from_slice(b"z00z.aead.v1\0");
    out.extend_from_slice(domain.as_bytes());
    out.push(0);
    out.extend_from_slice(context);
    out
}

fn aad_too_large(size: usize, limit: usize) -> CryptoError {
    CryptoError::AadTooLarge { size, limit }
}

fn aad_fixed_size(domain: &str, context_len: usize) -> Option<usize> {
    const PREFIX_LEN: usize = b"z00z.aead.v1\0".len();

    PREFIX_LEN
        .checked_add(domain.len())
        .and_then(|v| v.checked_add(1))
        .and_then(|v| v.checked_add(context_len))
}

fn ensure_aad_limit(total_size: usize, limit: usize) -> Result<(), CryptoError> {
    if total_size > limit {
        return Err(aad_too_large(total_size, limit));
    }
    Ok(())
}

fn multipart_context_len(parts: &[&[u8]], limit: usize) -> Result<usize, CryptoError> {
    parts.iter().try_fold(0usize, |ctx_len, part| {
        ctx_len
            .checked_add(8)
            .and_then(|v| v.checked_add(part.len()))
            .ok_or_else(|| aad_too_large(limit.saturating_add(1), limit))
    })
}

fn encode_multipart_context(parts: &[&[u8]], ctx_len: usize) -> Vec<u8> {
    let mut ctx = Vec::with_capacity(ctx_len);
    for part in parts {
        let part_len = (part.len() as u64).to_le_bytes();
        ctx.extend_from_slice(&part_len);
        ctx.extend_from_slice(part);
    }
    ctx
}

fn build_empty_aad(domain: &str, limit: usize) -> Result<Vec<u8>, CryptoError> {
    let total_size =
        aad_fixed_size(domain, 0).ok_or_else(|| aad_too_large(limit.saturating_add(1), limit))?;
    ensure_aad_limit(total_size, limit)?;
    Ok(build_aad(domain, b""))
}

fn build_single_aad(domain: &str, ctx: &[u8], limit: usize) -> Result<Vec<u8>, CryptoError> {
    let total_size = aad_fixed_size(domain, ctx.len())
        .ok_or_else(|| aad_too_large(limit.saturating_add(1), limit))?;
    ensure_aad_limit(total_size, limit)?;
    Ok(build_aad(domain, ctx))
}

fn build_multi_aad(domain: &str, parts: &[&[u8]], limit: usize) -> Result<Vec<u8>, CryptoError> {
    let ctx_len = multipart_context_len(parts, limit)?;
    let total_size = aad_fixed_size(domain, ctx_len)
        .ok_or_else(|| aad_too_large(limit.saturating_add(1), limit))?;
    ensure_aad_limit(total_size, limit)?;
    let ctx = encode_multipart_context(parts, ctx_len);
    Ok(build_aad(domain, &ctx))
}

fn build_aad_parts(domain: &str, parts: &[&[u8]], limit: usize) -> Result<Vec<u8>, CryptoError> {
    match parts {
        [] => build_empty_aad(domain, limit),
        [ctx] => build_single_aad(domain, ctx, limit),
        _ => build_multi_aad(domain, parts, limit),
    }
}

pub fn build_aad_multipart(domain: &str, parts: &[&[u8]]) -> Result<Vec<u8>, CryptoError> {
    build_aad_parts(domain, parts, MAX_AAD_SIZE)
}

#[doc(hidden)]
pub fn build_aad_multipart_extended(domain: &str, parts: &[&[u8]]) -> Result<Vec<u8>, CryptoError> {
    build_aad_parts(domain, parts, MAX_AAD_SIZE_EXTENDED)
}

pub(crate) fn validate_aad_size(aad: &[u8], limit: usize) -> Result<(), CryptoError> {
    if aad.len() > limit {
        return Err(CryptoError::AadTooLarge {
            size: aad.len(),
            limit,
        });
    }
    Ok(())
}
