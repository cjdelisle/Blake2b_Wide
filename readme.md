# Blake2b_Wide

SIMD version of Blake2b for performing multiple hashes at the same time.

Unlike
[other](https://github.com/oconnor663/blake2_simd)
[SIMD](https://github.com/minio/blake2b-simd)
[implementations](https://bench.cr.yp.to/impl-hash/blake2b.html) of Blake2b,
this is the simple/naive Blake2b algorithm, but performing multiple hashes at the same time.

This is potentially useful for constructs such as Merkle Trees, where you have a large
number of objects of the same length which must be hashed at the same time.

## Testing

Currently requires nightly for the new `std::simd` features.

```
cargo +nightly test
```

## Example

If you use `Blake2::<4>::default();` on AMD64 then you should expect it to generate AVX2
instructions. Using `Blake2::<8>::default();` will probably generate AVX-512 instructions.


```rust
let input = b"Example hash of blake2b.";
let mut input_long = mk_longs_from_bytes(&input[..]).unwrap();
let mut blake2b_4 = Blake2::<4>::default();
let mut hashes = [[0_u64;4]; 4];
blake2b_4.hash(
    hashes.chunks_mut(1).map(|a|&mut a[0][..]),
    [
        &input_long[..],
        &input_long[..],
        &input_long[..],
        &input_long[..],
    ],
);
let hash_result = "c4f1b39be6ee088be10fb0a016b3b9c2ebcf23d558c9410194467607a5da724d";
for h in hashes {
    assert_eq!(hash_result, hex::encode(mk_bytes_from_longs(h)));
}
```

## License
[LGPL-2.1-only](https://spdx.org/licenses/LGPL-2.1-only.html) OR
[LGPL-3.0-only](https://spdx.org/licenses/LGPL-3.0-only.html) OR
[Apache-2.0](https://spdx.org/licenses/Apache-2.0.html)