use std::fmt;

use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SecretBytes32(pub(crate) [u8; 32]);

impl SecretBytes32 {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn reveal(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn into_inner(self) -> [u8; 32] {
        let mut secret = self;
        std::mem::take(&mut secret.0)
    }
}

impl fmt::Debug for SecretBytes32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SecretBytes32(***)")
    }
}

impl subtle::ConstantTimeEq for SecretBytes32 {
    fn ct_eq(&self, other: &Self) -> subtle::Choice {
        self.0.ct_eq(&other.0)
    }
}

impl SecretBytes32 {
    pub fn ct_eq(&self, other: &Self) -> bool {
        use subtle::ConstantTimeEq;
        self.0.ct_eq(&other.0).into()
    }
}
