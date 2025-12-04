use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
/// A struct for make it clear that it has been hashed.
pub struct HashCell(u64);

impl HashCell {
    #[inline]
    /// Create new hash.
    pub fn new<H: Hash>(obj: H) -> Self {
        let mut hasher = DefaultHasher::new();

        obj.hash(&mut hasher);

        Self(hasher.finish())
    }

    #[inline]
    /// Combines hashes to Self with other.
    pub fn combine<H: Hash>(self, other: H) -> Self {
        let mut hasher = DefaultHasher::new();

        self.0.hash(&mut hasher);
        other.hash(&mut hasher);

        Self(hasher.finish())
    }
}

impl PartialEq<u64> for HashCell {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl PartialEq<HashCell> for u64 {
    fn eq(&self, other: &HashCell) -> bool {
        *self == other.0
    }
}

impl Debug for HashCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}
