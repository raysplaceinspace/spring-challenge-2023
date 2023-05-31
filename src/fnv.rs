/// We include the FNV hash because it is deterministic.
/// This is important if we are iterating over HashMaps or HashSets which don't have a deterministic iteration order in Rust.

use std::hash::{Hasher, BuildHasherDefault};
use std::collections::{HashMap, HashSet};

const INITIAL_STATE: u64 = 0xcbf29ce484222325;
const PRIME: u64 = 0x100000001b3;

pub struct FnvHasher(u64);

impl Default for FnvHasher {
    #[inline]
    fn default() -> FnvHasher {
        FnvHasher(INITIAL_STATE)
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let FnvHasher(mut hash) = *self;

        for byte in bytes.iter() {
            hash = hash ^ (*byte as u64);
            hash = hash.wrapping_mul(PRIME);
        }

        *self = FnvHasher(hash);
    }
}

pub type FnvBuildHasher = BuildHasherDefault<FnvHasher>;
pub type FnvHashMap<K, V> = HashMap<K, V, FnvBuildHasher>;
pub type FnvHashSet<T> = HashSet<T, FnvBuildHasher>;