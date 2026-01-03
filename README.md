# tempora-bloom

A lightweight, space-efficient Bloom filter implementation in Rust.

## What is a Bloom Filter?

A Bloom filter is a probabilistic data structure used to test whether an element is a member of a set. It is extremely space-efficient but allows for false positives. This means:

- If the filter says an item is **not present**, it is **definitely not present**.
- If the filter says an item **might be present**, it **could be a false positive**.

Bloom filters are useful when you need fast membership testing and can tolerate occasional false positives, such as in caching systems, spell checkers, or database query optimization.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tempora-bloom = "0.1.0"
```

## Usage

### Creating a Filter

Create a new Bloom filter by specifying the expected number of items and the desired false positive rate:

```rust
use tempora_bloom::StandardBloomFilter;

// Create a filter expecting 1000 items with a 1% false positive rate
let mut filter = StandardBloomFilter::new(1000, 0.01);
```

### Inserting Items

```rust
filter.insert("apple");
filter.insert("banana");
filter.insert(&42); // Works with any type that implements Hash
```

### Checking Membership

```rust
if filter.contains("apple") {
    println!("apple might be in the set");
}

if !filter.contains("orange") {
    println!("orange is definitely not in the set");
}
```

### Other Operations

```rust
// Get the size of the underlying bitmap in bits
let size = filter.len();

// Check if the filter has no bits set
let empty = filter.is_empty();

// Get the number of hash functions used
let k = filter.hash_count();

// Clear all items from the filter
filter.clear();
```

## API Reference

### `StandardBloomFilter::new(items_count: usize, fp_rate: f64)`

Creates a new Bloom filter optimized for the given parameters.

- `items_count`: Expected number of items to be inserted. Must be greater than 0.
- `fp_rate`: Desired false positive rate (e.g., 0.01 for 1%). Must be between 0 and 1 (exclusive).

The filter automatically calculates the optimal bitmap size and number of hash functions.

### `insert(&mut self, item: &T)`

Inserts an item into the filter. The item must implement the `Hash` trait.

### `contains(&self, item: &T) -> bool`

Returns `true` if the item might be in the filter, `false` if it is definitely not present.

### `clear(&mut self)`

Resets the filter by clearing all bits.

### `len(&self) -> usize`

Returns the size of the underlying bitmap in bits.

### `is_empty(&self) -> bool`

Returns `true` if no bits are set in the filter.

### `hash_count(&self) -> u32`

Returns the number of hash functions used by the filter.

## How It Works

This implementation uses enhanced double hashing to simulate multiple hash functions efficiently. Given two independent hash values h1 and h2, additional hash values are computed as:

```
h(i) = h1 + i * h2
```

The optimal bitmap size (m) and number of hash functions (k) are calculated based on the expected number of items (n) and desired false positive rate (p):

- Bitmap size: `m = -n * ln(p) / (ln(2)^2)`
- Hash functions: `k = -ln(p) / ln(2)`

## License

See LICENSE file for details.
