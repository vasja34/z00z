use super::{PaymentRequestError, RequestMetadata};

const STR_LEN_MAX: u32 = 1024;

#[derive(Clone, Debug)]
pub(super) struct ParsedRequest {
    pub(super) version: u8,
    pub(super) owner_handle: [u8; 32],
    pub(super) view_pk: [u8; 32],
    pub(super) identity_pk: [u8; 32],
    pub(super) req_id: [u8; 32],
    pub(super) chain_id: u32,
    pub(super) amount: Option<u64>,
    pub(super) expiry: u64,
    pub(super) metadata: Option<RequestMetadata>,
    pub(super) signature: [u8; 64],
}

impl ParsedRequest {
    pub(super) fn parse(bytes: &[u8]) -> Result<Self, PaymentRequestError> {
        let mut pos = 0usize;
        let head = parse_head(bytes, &mut pos)?;
        let body = parse_body(bytes, &mut pos)?;
        let signature = read_arr::<64>(bytes, &mut pos)?;

        if pos != bytes.len() {
            return Err(PaymentRequestError::InvalidRequestBytes);
        }

        Ok(Self {
            version: head.version,
            owner_handle: head.owner_handle,
            view_pk: head.view_pk,
            identity_pk: head.identity_pk,
            req_id: head.req_id,
            chain_id: head.chain_id,
            amount: body.amount,
            expiry: body.expiry,
            metadata: body.metadata,
            signature,
        })
    }
}

#[derive(Clone, Debug)]
struct ParsedHead {
    version: u8,
    owner_handle: [u8; 32],
    view_pk: [u8; 32],
    identity_pk: [u8; 32],
    req_id: [u8; 32],
    chain_id: u32,
}

fn parse_head(bytes: &[u8], pos: &mut usize) -> Result<ParsedHead, PaymentRequestError> {
    Ok(ParsedHead {
        version: read_u8(bytes, pos)?,
        owner_handle: read_arr::<32>(bytes, pos)?,
        view_pk: read_arr::<32>(bytes, pos)?,
        identity_pk: read_arr::<32>(bytes, pos)?,
        req_id: read_arr::<32>(bytes, pos)?,
        chain_id: read_u32(bytes, pos)?,
    })
}

#[derive(Clone, Debug)]
struct ParsedBody {
    amount: Option<u64>,
    expiry: u64,
    metadata: Option<RequestMetadata>,
}

fn parse_body(bytes: &[u8], pos: &mut usize) -> Result<ParsedBody, PaymentRequestError> {
    Ok(ParsedBody {
        amount: parse_opt_u64(bytes, pos)?,
        expiry: read_u64(bytes, pos)?,
        metadata: parse_opt_meta(bytes, pos)?,
    })
}

fn parse_meta(bytes: &[u8], pos: &mut usize) -> Result<RequestMetadata, PaymentRequestError> {
    let memo = parse_opt_string(bytes, pos)?;
    let payment_id = parse_opt_payment_id(bytes, pos)?;
    let min_confirmations = parse_opt_u32(bytes, pos)?;
    let return_receiver = parse_opt_string(bytes, pos)?;
    let created_at = read_u64(bytes, pos)?;

    Ok(RequestMetadata {
        memo,
        payment_id,
        min_confirmations,
        return_receiver,
        created_at,
    })
}

fn parse_opt_string(bytes: &[u8], pos: &mut usize) -> Result<Option<String>, PaymentRequestError> {
    let flag = read_u8(bytes, pos)?;
    if flag == 0 {
        return Ok(None);
    }
    if flag != 1 {
        return Err(PaymentRequestError::InvalidRequestFlag);
    }

    let len = read_u32(bytes, pos)?;
    if len > STR_LEN_MAX {
        return Err(PaymentRequestError::InvalidRequestSize);
    }

    let value = read_utf8(bytes, pos, len)?;
    Ok(Some(value))
}

fn parse_opt_u64(bytes: &[u8], pos: &mut usize) -> Result<Option<u64>, PaymentRequestError> {
    parse_opt(bytes, pos, read_u64)
}

fn parse_opt_u32(bytes: &[u8], pos: &mut usize) -> Result<Option<u32>, PaymentRequestError> {
    parse_opt(bytes, pos, read_u32)
}

fn parse_opt_payment_id(
    bytes: &[u8],
    pos: &mut usize,
) -> Result<Option<[u8; 16]>, PaymentRequestError> {
    parse_opt(bytes, pos, read_arr::<16>)
}

fn parse_opt_meta(
    bytes: &[u8],
    pos: &mut usize,
) -> Result<Option<RequestMetadata>, PaymentRequestError> {
    parse_opt(bytes, pos, parse_meta)
}

fn parse_opt<T>(
    bytes: &[u8],
    pos: &mut usize,
    parser: fn(&[u8], &mut usize) -> Result<T, PaymentRequestError>,
) -> Result<Option<T>, PaymentRequestError> {
    let flag = read_u8(bytes, pos)?;
    if flag == 0 {
        return Ok(None);
    }
    if flag == 1 {
        return parser(bytes, pos).map(Some);
    }
    Err(PaymentRequestError::InvalidRequestFlag)
}

fn read_utf8(bytes: &[u8], pos: &mut usize, len: u32) -> Result<String, PaymentRequestError> {
    let chunk = read_chunk(bytes, pos, len)?;
    let value =
        std::str::from_utf8(chunk).map_err(|_| PaymentRequestError::InvalidRequestString)?;
    Ok(value.to_string())
}

fn read_chunk<'a>(
    bytes: &'a [u8],
    pos: &mut usize,
    len: u32,
) -> Result<&'a [u8], PaymentRequestError> {
    let len_usize = usize::try_from(len).map_err(|_| PaymentRequestError::InvalidRequestSize)?;
    let end = pos
        .checked_add(len_usize)
        .ok_or(PaymentRequestError::InvalidRequestSize)?;
    if end > bytes.len() {
        return Err(PaymentRequestError::InvalidRequestBytes);
    }

    let chunk = &bytes[*pos..end];
    *pos = end;
    Ok(chunk)
}

fn read_u8(bytes: &[u8], pos: &mut usize) -> Result<u8, PaymentRequestError> {
    if *pos >= bytes.len() {
        return Err(PaymentRequestError::InvalidRequestBytes);
    }
    let value = bytes[*pos];
    *pos += 1;
    Ok(value)
}

fn read_u32(bytes: &[u8], pos: &mut usize) -> Result<u32, PaymentRequestError> {
    let value = read_arr::<4>(bytes, pos)?;
    Ok(u32::from_le_bytes(value))
}

fn read_u64(bytes: &[u8], pos: &mut usize) -> Result<u64, PaymentRequestError> {
    let value = read_arr::<8>(bytes, pos)?;
    Ok(u64::from_le_bytes(value))
}

fn read_arr<const N: usize>(bytes: &[u8], pos: &mut usize) -> Result<[u8; N], PaymentRequestError> {
    let end = pos
        .checked_add(N)
        .ok_or(PaymentRequestError::InvalidRequestSize)?;
    if end > bytes.len() {
        return Err(PaymentRequestError::InvalidRequestBytes);
    }
    let part = bytes[*pos..end]
        .try_into()
        .map_err(|_| PaymentRequestError::InvalidRequestBytes)?;
    *pos = end;
    Ok(part)
}
