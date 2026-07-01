use subtle::ConstantTimeEq;

use super::compute_owner_tag;

/// Owner tag wrapper with verification helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnerTag {
    tag: [u8; 32],
}

impl OwnerTag {
    /// Compute owner tag from owner handle and shared key.
    pub fn compute(owner_handle: &[u8; 32], k_dh: &[u8; 32]) -> Self {
        Self {
            tag: compute_owner_tag(owner_handle, k_dh),
        }
    }

    /// Verify against external bytes in constant time.
    pub fn verify(&self, other: &[u8; 32]) -> bool {
        self.tag.ct_eq(other).into()
    }

    /// Return owner tag bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.tag
    }
}

/// Trait for owner tag handling over keying material.
pub trait OwnerTagOperations {
    /// Compute owner tag for current owner handle.
    fn compute_tag(&self, k_dh: &[u8; 32]) -> OwnerTag;

    /// Verify owner tag for current owner handle and key.
    fn verify_tag(&self, tag: &[u8; 32], k_dh: &[u8; 32]) -> bool;
}

impl OwnerTagOperations for [u8; 32] {
    fn compute_tag(&self, k_dh: &[u8; 32]) -> OwnerTag {
        OwnerTag::compute(self, k_dh)
    }

    fn verify_tag(&self, tag: &[u8; 32], k_dh: &[u8; 32]) -> bool {
        self.compute_tag(k_dh).verify(tag)
    }
}

#[cfg(test)]
mod tests {
    use super::{OwnerTag, OwnerTagOperations};

    #[test]
    fn test_owner_tag_struct() {
        let owner = [1u8; 32];
        let k_dh = [2u8; 32];
        let tag = OwnerTag::compute(&owner, &k_dh);

        assert_eq!(tag.as_bytes().len(), 32);
        assert!(tag.verify(tag.as_bytes()));
    }

    #[test]
    fn test_owner_tag_ops() {
        let owner = [3u8; 32];
        let k_dh = [4u8; 32];
        let other = [5u8; 32];

        let tag = owner.compute_tag(&k_dh);
        assert!(owner.verify_tag(tag.as_bytes(), &k_dh));
        assert!(!other.verify_tag(tag.as_bytes(), &k_dh));
    }
}
