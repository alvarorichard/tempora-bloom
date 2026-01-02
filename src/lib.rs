use bit_vec::BitVec;
use std::collections::hash_map::{DefaultHasher, RandomState};
use std::hash::{BuildHasher, Hash, Hasher};
use std::marker::PhantomData;

/// A standard Bloom filter implementation.
///
/// A Bloom filter is a space-efficient probabilistic data structure that is used
/// to test whether an element is a member of a set. False positive matches are
/// possible, but false negatives are not.
///
/// # Type Parameters
/// - `T`: The type of items to store. Must implement `Hash`.
///
/// # Example
/// ```
/// use tempora_bloom::StandardBloomFilter;
///
/// let mut filter = StandardBloomFilter::new(1000, 0.01);
/// filter.insert("hello");
/// assert!(filter.contains("hello"));
/// assert!(!filter.contains("world")); // Might be true due to false positive
/// ```
pub struct StandardBloomFilter<T: ?Sized + Hash> {
    bitmap: BitVec,
    optimal_k: u32,
    hashers: [DefaultHasher; 2],
    _marker: PhantomData<T>,
}

impl<T: ?Sized + Hash> StandardBloomFilter<T> {
    /// Creates a new Bloom filter optimized for the expected number of items
    /// and desired false positive rate.
    ///
    /// # Arguments
    /// * `items_count` - Expected number of items to be inserted
    /// * `fp_rate` - Desired false positive rate (e.g., 0.01 for 1%)
    ///
    /// # Panics
    /// Panics if `fp_rate` is not in the range (0, 1) or if `items_count` is 0.
    pub fn new(items_count: usize, fp_rate: f64) -> Self {
        assert!(items_count > 0, "items_count must be greater than 0");
        assert!(
            fp_rate > 0.0 && fp_rate < 1.0,
            "fp_rate must be between 0 and 1 (exclusive)"
        );

        let optimal_m = Self::bitmap_size(items_count, fp_rate);
        let optimal_k = Self::optimal_k(fp_rate);
        let hashers = [
            RandomState::new().build_hasher(),
            RandomState::new().build_hasher(),
        ];

        Self {
            bitmap: BitVec::from_elem(optimal_m, false),
            optimal_k,
            hashers,
            _marker: PhantomData,
        }
    }

    /// Calculates the optimal bitmap size based on expected items and false positive rate.
    fn bitmap_size(items_count: usize, fp_rate: f64) -> usize {
        let ln2_2 = core::f64::consts::LN_2.powi(2);
        ((-1.0f64 * items_count as f64 * fp_rate.ln()) / ln2_2).ceil() as usize
    }

    /// Calculates the optimal number of hash functions based on false positive rate.
    fn optimal_k(fp_rate: f64) -> u32 {
        (-(fp_rate.ln() / core::f64::consts::LN_2)).ceil() as u32
    }

    /// Inserts an item into the Bloom filter.
    ///
    /// # Arguments
    /// * `item` - The item to insert
    pub fn insert(&mut self, item: &T) {
        let (h1, h2) = self.hash_kernel(item);
        let len = self.bitmap.len();

        for k_i in 0..self.optimal_k {
            let index = Self::get_index(h1, h2, k_i as u64, len);
            self.bitmap.set(index, true);
        }
    }

    /// Checks if an item might be in the Bloom filter.
    ///
    /// # Arguments
    /// * `item` - The item to check
    ///
    /// # Returns
    /// * `true` if the item might be in the filter (can be a false positive)
    /// * `false` if the item is definitely not in the filter
    pub fn contains(&self, item: &T) -> bool {
        let (h1, h2) = self.hash_kernel(item);
        let len = self.bitmap.len();

        for k_i in 0..self.optimal_k {
            let index = Self::get_index(h1, h2, k_i as u64, len);
            if !self.bitmap.get(index).unwrap_or(false) {
                return false;
            }
        }

        true
    }

    /// Clears all items from the Bloom filter.
    pub fn clear(&mut self) {
        self.bitmap.clear();
    }

    /// Returns the size of the underlying bitmap in bits.
    pub fn len(&self) -> usize {
        self.bitmap.len()
    }

    /// Returns `true` if the filter has no bits set.
    pub fn is_empty(&self) -> bool {
        self.bitmap.none()
    }

    /// Returns the number of hash functions used.
    pub fn hash_count(&self) -> u32 {
        self.optimal_k
    }

    /// Computes two independent hash values for the given item.
    fn hash_kernel(&self, item: &T) -> (u64, u64) {
        let mut hasher1 = self.hashers[0].clone();
        item.hash(&mut hasher1);
        let h1 = hasher1.finish();

        let mut hasher2 = self.hashers[1].clone();
        item.hash(&mut hasher2);
        let h2 = hasher2.finish();

        (h1, h2)
    }

    /// Computes the bit index using enhanced double hashing.
    #[inline]
    fn get_index(h1: u64, h2: u64, k_i: u64, len: usize) -> usize {
        let combined_hash = h1.wrapping_add(k_i.wrapping_mul(h2));
        (combined_hash % len as u64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut bloom = StandardBloomFilter::new(100, 0.01);
        bloom.insert("item");
        assert!(bloom.contains("item"));
    }

    #[test]
    fn test_not_contains_before_insert() {
        let mut bloom = StandardBloomFilter::new(100, 0.01);
        assert!(!bloom.contains("item_1"));
        assert!(!bloom.contains("item_2"));
        bloom.insert("item_1");
        assert!(bloom.contains("item_1"));
    }

    #[test]
    fn test_clear() {
        let mut bloom = StandardBloomFilter::new(100, 0.01);
        bloom.insert("item");
        assert!(bloom.contains("item"));
        bloom.clear();
        assert!(bloom.is_empty());
    }

    #[test]
    fn test_multiple_items() {
        let mut bloom = StandardBloomFilter::new(1000, 0.01);
        let items = ["apple", "banana", "cherry", "date", "elderberry"];

        for item in &items {
            bloom.insert(*item);
        }

        for item in &items {
            assert!(bloom.contains(*item));
        }
    }

    #[test]
    fn test_integer_items() {
        let mut bloom: StandardBloomFilter<i32> = StandardBloomFilter::new(100, 0.01);
        bloom.insert(&42);
        bloom.insert(&123);
        assert!(bloom.contains(&42));
        assert!(bloom.contains(&123));
    }

    #[test]
    #[should_panic(expected = "items_count must be greater than 0")]
    fn test_zero_items_count_panics() {
        let _bloom: StandardBloomFilter<str> = StandardBloomFilter::new(0, 0.01);
    }

    #[test]
    #[should_panic(expected = "fp_rate must be between 0 and 1")]
    fn test_invalid_fp_rate_panics() {
        let _bloom: StandardBloomFilter<str> = StandardBloomFilter::new(100, 1.5);
    }
}
