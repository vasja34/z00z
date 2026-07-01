//! NFC helpers for payment request exchange.

use crate::receiver::PaymentRequest;

/// Creates NFC NDEF URI record bytes from a payment request.
pub fn nfc_ndef_record(request: &PaymentRequest) -> Vec<u8> {
    let uri = format!("z00z:pay?{}", request.to_qr_code_data());
    create_ndef_uri_record(&uri)
}

fn create_ndef_uri_record(uri: &str) -> Vec<u8> {
    let uri_bytes = uri.as_bytes();
    let payload_len = uri_bytes.len().saturating_add(1);

    if payload_len <= usize::from(u8::MAX) {
        let mut record = vec![0xD1, 0x01, payload_len as u8, 0x55, 0x00];
        record.extend_from_slice(uri_bytes);
        return record;
    }

    let mut record = vec![0xC1, 0x01];
    record.extend_from_slice(&(payload_len as u32).to_be_bytes());
    record.push(0x55);
    record.push(0x00);
    record.extend_from_slice(uri_bytes);
    record
}

#[cfg(test)]
mod tests {
    use super::nfc_ndef_record;
    use crate::receiver::PaymentRequest;

    fn sample_req() -> PaymentRequest {
        PaymentRequest {
            version: 1,
            owner_handle: [1u8; 32],
            view_pk: [2u8; 32],
            identity_pk: [3u8; 32],
            req_id: [4u8; 32],
            chain_id: 1,
            amount: Some(77),
            expiry: u64::MAX,
            metadata: None,
            signature: [0u8; 64],
        }
    }

    #[test]
    fn test_nfc_ndef_record_encoding() {
        let request = sample_req();
        let record = nfc_ndef_record(&request);

        assert!(record.len() > 3);
        assert_eq!(record[1], 0x01);

        let is_short = (record[0] & 0x10) != 0;
        let (payload_len, payload_idx) = if is_short {
            (usize::from(record[2]), 4usize)
        } else {
            let mut len_bytes = [0u8; 4];
            len_bytes.copy_from_slice(&record[2..6]);
            (u32::from_be_bytes(len_bytes) as usize, 7usize)
        };

        assert_eq!(payload_len, record.len() - payload_idx);
        assert_eq!(record[payload_idx - 1], 0x55);
        assert_eq!(record[payload_idx], 0x00);

        let payload = std::str::from_utf8(&record[payload_idx + 1..]).expect("utf8 payload");
        assert!(payload.starts_with("z00z:pay?"));
    }
}
