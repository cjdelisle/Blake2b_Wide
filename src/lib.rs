#![feature(portable_simd)]
use std::simd::{Simd, SupportedLaneCount, LaneCount};
use std::convert::TryInto;

const IV: [u64; 8] = [
    0x6a09e667f3bcc908, 0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
    0x510e527fade682d1, 0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
];

const SIGMA: [[usize; 16]; 12] = [
    [  0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15 ],
    [ 14, 10,  4,  8,  9, 15, 13,  6,  1, 12,  0,  2, 11,  7,  5,  3 ],
    [ 11,  8, 12,  0,  5,  2, 15, 13, 10, 14,  3,  6,  7,  1,  9,  4 ],
    [  7,  9,  3,  1, 13, 12, 11, 14,  2,  6,  5, 10,  4,  0, 15,  8 ],
    [  9,  0,  5,  7,  2,  4, 10, 15, 14,  1, 11, 12,  6,  8,  3, 13 ],
    [  2, 12,  6, 10,  0, 11,  8,  3,  4, 13,  7,  5, 15, 14,  1,  9 ],
    [ 12,  5,  1, 15, 14, 13,  4, 10,  0,  7,  6,  3,  9,  2,  8, 11 ],
    [ 13, 11,  7, 14, 12,  1,  3,  9,  5,  0, 15,  4,  8,  6,  2, 10 ],
    [  6, 15, 14,  9, 11,  3,  0,  8, 12,  2, 13,  7,  1,  4, 10,  5 ],
    [ 10,  2,  8,  4,  7,  6,  1,  5, 15, 11,  9, 14,  3, 12, 13,  0 ],
    [  0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15 ],
    [ 14, 10,  4,  8,  9, 15, 13,  6,  1, 12,  0,  2, 11,  7,  5,  3 ],
];

macro_rules! rotr64 {
    ($v:expr, $n:expr) => {
        ($v >> $n) |  ($v << (64 - $n))
    }
}

macro_rules! blake2_g {
    ($a:ident, $b:ident, $c:ident, $d:ident, $x:expr, $y:expr) => {
        $a += $b + $x; $d = rotr64!($d ^ $a, 32);
        $c += $d;      $b = rotr64!($b ^ $c, 24);
        $a += $b + $y; $d = rotr64!($d ^ $a, 16);
        $c += $d;      $b = rotr64!($b ^ $c, 63);
    }
}

type Wu64<const N: usize> = Simd<u64, N>;

// These are not implemented
const HASH_SZ: u64 = 32;
const KEY_SZ: u64 = 0;

#[derive(Default)]
pub struct Blake2<const N: usize>
    where LaneCount<N>: SupportedLaneCount
{
    hash: [Wu64<N>; 8],
}
impl<const N: usize> Blake2<N>
    where LaneCount<N>: SupportedLaneCount
{
    fn compute(&mut self, input: &[Wu64<N>; 16], input_offset: u64, is_last_block: bool) {

        let mut v0 = self.hash[0];
        let mut v1 = self.hash[1];
        let mut v2 = self.hash[2];
        let mut v3 = self.hash[3];
        let mut v4 = self.hash[4];
        let mut v5 = self.hash[5];
        let mut v6 = self.hash[6];
        let mut v7 = self.hash[7];

        let mut v8 = Wu64::<N>::splat(IV[0]);
        let mut v9 = Wu64::<N>::splat(IV[1]);
        let mut v10 = Wu64::<N>::splat(IV[2]);
        let mut v11 = Wu64::<N>::splat(IV[3]);
        let mut v12 = Wu64::<N>::splat(IV[4] ^ input_offset);
        let mut v13 = Wu64::<N>::splat(IV[5]);
        let mut v14 = Wu64::<N>::splat(IV[6] ^ if is_last_block { u64::MAX } else { 0 });
        let mut v15 = Wu64::<N>::splat(IV[7]);

        macro_rules! blake2_round {
            ($i:expr) => {
                blake2_g!(v0, v4, v8 , v12, input[SIGMA[$i][ 0]], input[SIGMA[$i][ 1]]);
                blake2_g!(v1, v5, v9 , v13, input[SIGMA[$i][ 2]], input[SIGMA[$i][ 3]]);
                blake2_g!(v2, v6, v10, v14, input[SIGMA[$i][ 4]], input[SIGMA[$i][ 5]]);
                blake2_g!(v3, v7, v11, v15, input[SIGMA[$i][ 6]], input[SIGMA[$i][ 7]]);
                blake2_g!(v0, v5, v10, v15, input[SIGMA[$i][ 8]], input[SIGMA[$i][ 9]]);
                blake2_g!(v1, v6, v11, v12, input[SIGMA[$i][10]], input[SIGMA[$i][11]]);
                blake2_g!(v2, v7, v8 , v13, input[SIGMA[$i][12]], input[SIGMA[$i][13]]);
                blake2_g!(v3, v4, v9 , v14, input[SIGMA[$i][14]], input[SIGMA[$i][15]]);
            }
        }
        blake2_round!(0);
        blake2_round!(1);
        blake2_round!(2);
        blake2_round!(3);
        blake2_round!(4);
        blake2_round!(5);
        blake2_round!(6);
        blake2_round!(7);
        blake2_round!(8);
        blake2_round!(9);
        blake2_round!(10);
        blake2_round!(11);

        // update hash
        self.hash[0] ^= v0 ^ v8;   self.hash[1] ^= v1 ^ v9;
        self.hash[2] ^= v2 ^ v10;  self.hash[3] ^= v3 ^ v11;
        self.hash[4] ^= v4 ^ v12;  self.hash[5] ^= v5 ^ v13;
        self.hash[6] ^= v6 ^ v14;  self.hash[7] ^= v7 ^ v15;
    }

    fn init(&mut self) {
        for (h, iv) in self.hash.iter_mut().zip(IV) {
            *h = Wu64::<N>::splat(iv);
        }
        self.hash[0] ^= Wu64::<N>::splat(0x01010000 ^ (KEY_SZ << 8) ^ HASH_SZ);
    }

    pub fn hash<'a>(&mut self, mut out: impl Iterator<Item = &'a mut [u64]>, inputs: [&[u64]; N]) {
        self.init();
        {
            let mut inp = [Wu64::<N>::default(); 16];
            let inlen = inputs[0].len();
            let len = (inlen / 16 + if (inlen % 16) != 0 { 1 } else { 0 }) * 16;
            //println!("len={}", len);
            for i in 0..len {
                for j in 0..N {
                    if i < inlen {
                        inp[i % 16][j] = inputs[j][i];
                    } else {
                        inp[i % 16][j] = 0;
                    }
                }
                if i % 16 == 15 {
                    self.compute(&inp, std::cmp::min(i + 1, inlen) as u64 * 8, i == len - 1);
                }
            }
        }
        for j in 0..N {
            let n = out.next().unwrap();
            for i in 0..4 {
                n[i] = self.hash[i][j];
            }
        }
    }

    // pub fn output() -> Blake2HashN<N> {
    //     Blake2HashN([Blake2Hash::default(); N])
    // }
}

pub fn longs_from_bytes(out: &mut [u64], b: &[u8]) -> std::result::Result<(), &'static str> {
    if out.len() * 8 != b.len() {
        Err("length mismatch")
    } else {
        for (o, i) in out.iter_mut().zip((0..).step_by(8)) {
            *o = u64::from_le_bytes((&b[i..i+8]).try_into().unwrap())
        }
        Ok(())
    }
}

pub fn mk_longs_from_bytes(b: &[u8]) -> std::result::Result<Vec<u64>, &'static str> {
    if b.len() % 8 != 0 {
        return Err("length of byte array must be a multiple of 8");
    }
    let mut out = vec![0_u64; b.len() / 8];
    longs_from_bytes(&mut out, b).unwrap();
    Ok(out)
}

pub fn bytes_from_longs(out: &mut [u8], l: &[u64]) -> std::result::Result<(), &'static str> {
    if out.len() != l.len() * 8 {
        Err("length mismatch")
    } else {
        for (l, i) in l.iter().zip((0..).step_by(8)) {
            out[i..i+8].copy_from_slice(&l.to_le_bytes()[..]);
        }
        Ok(())
    }
}

pub fn mk_bytes_from_longs(l: &[u64]) -> Vec<u8> {
    let mut out = vec![0_u8; l.len() * 8];
    bytes_from_longs(&mut out, l).unwrap();
    out
}

#[cfg(tests)]
mod tests {
    #[test]
    fn test_long_hash() {
        let input = mk_longs_from_bytes(&[0_u8; 2048]);
        let mut blake2b_4 = Blake2::<4>::default();
        let mut hashes = [[0_u64;4]; 4];
        blake2b_4.hash(
            hashes.chunks_mut(1).map(|a|&mut a[0][..]),
            [
                &input[..],
                &input[..],
                &input[..],
                &input[..],
            ],
        );
        let expected_result = "200823e5158b3774c11b5c61850ada762f8264144a9bebec3ebac5a2adde67b8";
        for h in hashes {
            assert_eq!(expected_result, hex::encode(mk_bytes_from_longs(h)));
        }
    }

    #[test]
    fn test_short_hash() {
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
    }
}