#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{
        fs,
        io::{self, BufRead},
        path::PathBuf,
    };

    use z00z_wallets::domains::hashing::compute_password_bloom;

    const M_BITS: u64 = 8 * 1024 * 1024;
    const K: u64 = 7;

    fn h64(prefix: u8, input: &[u8]) -> Result<u64, Box<dyn std::error::Error>> {
        let out = compute_password_bloom(prefix, input);
        let bytes: [u8; 8] = out[0..8]
            .try_into()
            .map_err(|_| "unexpected digest length")?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn set_bit(bits: &mut [u8], bit_index: u64) {
        let byte_index = (bit_index / 8) as usize;
        let mask = 1u8 << (bit_index % 8);
        bits[byte_index] |= mask;
    }

    fn add(bits: &mut [u8], word: &str) -> Result<(), Box<dyn std::error::Error>> {
        let input = word.as_bytes();
        let h1 = h64(0, input)?;
        let h2 = h64(1, input)?;

        for i in 0..K {
            let idx = h1.wrapping_add(i.wrapping_mul(h2)) % M_BITS;
            set_bit(bits, idx);
        }

        Ok(())
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src/config/common-passwords.txt");
    let out = root.join("src/config/password_denylist.bloom");

    let file = fs::File::open(&src)?;
    let reader = io::BufReader::new(file);

    let mut bits = vec![0u8; (M_BITS / 8) as usize];
    let mut n = 0usize;

    for line in reader.lines() {
        let line = line?;
        let word = line.trim();
        if word.is_empty() {
            continue;
        }

        add(&mut bits, &word.to_ascii_lowercase())?;
        n += 1;
    }

    let Some(parent) = out.parent() else {
        return Err("output path has no parent directory".into());
    };
    fs::create_dir_all(parent)?;
    fs::write(&out, &bits)?;

    println!(
        "Wrote {} bytes to {} from {} words (M_BITS={}, K={})",
        bits.len(),
        out.display(),
        n,
        M_BITS,
        K
    );

    Ok(())
}
