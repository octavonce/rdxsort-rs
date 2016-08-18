extern crate rand;
extern crate rdxsort;

use std::collections;
use std::hash;
use std::hash::Hash;
use std::hash::Hasher;
use std::iter;
use std::mem;
use std::ops;

use rand::{Rng, XorShiftRng};
use rand::distributions::{IndependentSample, Range};
use rand::distributions::range::SampleRange;

use rdxsort::*;

pub static CFG_N: usize = 10_000;

fn is_sorted<T>(data: &Vec<T>) -> bool
    where T: Clone,
          T: PartialOrd
{
    let mut last_entry: Option<T> = None;
    for x in data.iter().cloned() {
        match last_entry {
            Some(l) => {
                if !(l.le(&x)) {
                    return false;
                }
            }
            None => {}
        }
        last_entry = Some(x);
    }
    return true;
}

pub trait MyHash {
    fn hash_it<H>(&self, state: &mut H) where H: Hasher;
}

macro_rules! trivial_myhash {
    ($t:ty) => {
        impl MyHash for $t {
            fn hash_it<H>(&self, state: &mut H) where H: Hasher {
                self.hash(state);
            }
        }
    }
}

trivial_myhash!(i8);
trivial_myhash!(i16);
trivial_myhash!(i32);
trivial_myhash!(i64);
trivial_myhash!(u8);
trivial_myhash!(u16);
trivial_myhash!(u32);
trivial_myhash!(u64);

impl MyHash for f32 {
    fn hash_it<H>(&self, state: &mut H) where H: Hasher {
        let alias = unsafe {mem::transmute_copy::<f32, u32>(self)};
        alias.hash(state);
    }
}

impl MyHash for f64 {
    fn hash_it<H>(&self, state: &mut H) where H: Hasher {
        let alias = unsafe {mem::transmute_copy::<f64, u64>(self)};
        alias.hash(state);
    }
}

fn guess_entropy<T>(data: &Vec<T>) -> f64 where T: MyHash {
    let mut counter = collections::HashMap::<u64, usize>::new();

    for x in data {
        let mut hasher = hash::SipHasher::new();
        x.hash_it(&mut hasher);
        let key = hasher.finish();

        let state = counter.entry(key).or_insert(0);
        *state += 1;
    }

    counter.values().map(|&x| x as f64 / data.len() as f64).map(|x| -x * x.log2()).fold(0f64, |a, b| a + b)
}

pub fn test_generic<T>(data: Vec<T>)
    where T: Clone + PartialOrd,
          Vec<T>: RdxSort
{
    let mut data = data;
    let n = data.len();
    let mut data_sorted_ref = data.clone();
    data_sorted_ref.sort_by(|a, b| a.partial_cmp(b).unwrap());

    data.rdxsort();
    assert!(data.len() == n, "sorted data has wrong lenght!");
    assert!(is_sorted(&data), "data is not sorted!");
    for (x, y) in data.iter().zip(data_sorted_ref.iter()) {
        assert!(x == y, "sortd data does not match the reference!");
    }
}

pub fn test_empty_generic<T>()
    where T: Clone + PartialOrd,
          Vec<T>: RdxSort
{
    let data: Vec<T> = vec![];
    test_generic(data);
}

pub fn test_rnd_generic<T>(vmin: T, vmax: T, vspecial: Vec<T>)
    where T: Clone + PartialOrd + SampleRange + MyHash,
          Vec<T>: RdxSort
{
    // config
    let entropy_threshold = 1f64;

    // generate data
    let r = Range::new(vmin.clone(), vmax.clone());
    let mut rng = XorShiftRng::new_unseeded();
    let mut data = Vec::with_capacity(CFG_N);
    for _ in (0 as usize)..CFG_N {
        data.push(r.ind_sample(&mut rng));
    }
    let mut positions = Vec::with_capacity(CFG_N);
    for i in 0..CFG_N {
        positions.push(i);
    }
    rng.shuffle(&mut positions[..]);
    assert!(vspecial.len() + 2 < CFG_N, "to many special values to test!");
    data[positions[0]] = vmin;
    data[positions[1]] = vmax;
    for (i, x) in positions.into_iter().skip(2).zip(vspecial.into_iter()) {
        data[i] = x;
    }
    assert!(data.len() == CFG_N, "generated data has wrong length!");
    assert!(guess_entropy(&data) >= entropy_threshold, "generated data does not contain enough entropy!");

    test_generic(data);
}

pub fn test_single_generic<T>(x: T)
    where T: Clone + PartialOrd,
          Vec<T>: RdxSort
{
    let data: Vec<T> = vec![x];
    test_generic(data);
}

pub fn test_full_generic<T>(vmin: T, vmax: T)
    where T: Clone+ PartialOrd + ops::Add,
          ops::Range<T>: iter::Iterator,
          Vec<T>: iter::FromIterator<<ops::Range<T> as iter::Iterator>::Item>,
          Vec<T>: RdxSort
{
    // TODO: use inclusive range once it's stable
    let mut data: Vec<T> = (vmin.clone()..vmax.clone()).collect();
    data.push(vmax);

    let mut rng = XorShiftRng::new_unseeded();
    rng.shuffle(&mut data[..]);

    test_generic(data);
}

mod sub_bool {
    use super::*;

    use rand::{Rng, XorShiftRng};

    #[test]
    fn test_rnd_bool() {
        let mut rng = XorShiftRng::new_unseeded();
        let data: Vec<bool> = rng.gen_iter::<bool>().take(CFG_N).collect();

        test_generic(data);
    }

    #[test]
    fn test_empty_bool() {
        test_empty_generic::<bool>();
    }

    #[test]
    fn test_single_bool() {
        test_single_generic::<bool>(true);
    }
}

mod sub_i8 {
    use super::*;

    #[test]
    fn test_rnd_i8() {
        test_rnd_generic::<i8>(i8::min_value(), i8::max_value(), vec![i8::min_value() + 1, -1i8, 0i8, 1i8, i8::max_value() - 1]);
    }

    #[test]
    fn test_full_i8() {
        test_full_generic::<i8>(i8::min_value(), i8::max_value());
    }

    #[test]
    fn test_empty_i8() {
        test_empty_generic::<i8>();
    }

    #[test]
    fn test_single_i8() {
        test_single_generic::<i8>(3i8);
    }
}

mod sub_i16 {
    use super::*;

    #[test]
    fn test_rnd_i16() {
        test_rnd_generic::<i16>(i16::min_value(), i16::max_value(), vec![i16::min_value() + 1, -1i16, 0i16, 1i16, i16::max_value() - 1]);
    }

    #[test]
    fn test_full_i16() {
        test_full_generic::<i16>(i16::min_value(), i16::max_value());
    }

    #[test]
    fn test_empty_i16() {
        test_empty_generic::<i16>();
    }

    #[test]
    fn test_single_i16() {
        test_single_generic::<i16>(3i16);
    }
}

mod sub_i32 {
    use super::*;

    #[test]
    fn test_rnd_i32() {
        test_rnd_generic::<i32>(i32::min_value(), i32::max_value(), vec![i32::min_value() + 1, -1i32, 0i32, 1i32, i32::max_value() - 1]);
    }

    #[test]
    fn test_empty_i32() {
        test_empty_generic::<i32>();
    }

    #[test]
    fn test_single_i32() {
        test_single_generic::<i32>(3i32);
    }
}

mod sub_i64 {
    use super::*;

    #[test]
    fn test_rnd_i64() {
        test_rnd_generic::<i64>(i64::min_value(), i64::max_value(), vec![i64::min_value() + 1, -1i64, 0i64, 1i64, i64::max_value() - 1]);
    }

    #[test]
    fn test_empty_i64() {
        test_empty_generic::<i64>();
    }

    #[test]
    fn test_single_i64() {
        test_single_generic::<i64>(3i64);
    }
}

mod sub_u8 {
    use super::*;

    #[test]
    fn test_rnd_u8() {
        test_rnd_generic::<u8>(u8::min_value(), u8::max_value(), vec![0u8, 1u8, u8::max_value() - 1]);
    }

    #[test]
    fn test_full_u8() {
        test_full_generic::<u8>(u8::min_value(), u8::max_value());
    }

    #[test]
    fn test_empty_u8() {
        test_empty_generic::<u8>();
    }

    #[test]
    fn test_single_u8() {
        test_single_generic::<u8>(3u8);
    }
}

mod sub_u16 {
    use super::*;

    #[test]
    fn test_rnd_u16() {
        test_rnd_generic::<u16>(u16::min_value(), u16::max_value(), vec![0u16, 1u16, u16::max_value() - 1]);
    }

    #[test]
    fn test_full_u16() {
        test_full_generic::<u16>(u16::min_value(), u16::max_value());
    }

    #[test]
    fn test_empty_u16() {
        test_empty_generic::<u16>();
    }

    #[test]
    fn test_single_u16() {
        test_single_generic::<u16>(3u16);
    }
}

mod sub_u32 {
    use super::*;

    #[test]
    fn test_rnd_u32() {
        test_rnd_generic::<u32>(u32::min_value(), u32::max_value(), vec![0u32, 1u32, u32::max_value() - 1]);
    }

    #[test]
    fn test_empty_u32() {
        test_empty_generic::<u32>();
    }

    #[test]
    fn test_single_u32() {
        test_single_generic::<u32>(3u32);
    }
}

mod sub_u64 {
    use super::*;

    #[test]
    fn test_rnd_u64() {
        test_rnd_generic::<u64>(u64::min_value(), u64::max_value(), vec![0u64, 1u64, u64::max_value() - 1]);
    }

    #[test]
    fn test_empty_u64() {
        test_empty_generic::<u64>();
    }

    #[test]
    fn test_single_u64() {
        test_single_generic::<u64>(3u64);
    }
}

mod sub_f32 {
    use super::*;

    use std::f32;

    #[test]
    fn test_rnd_f32() {
        // DO NOT use MIN/MAX here, since that overflows the RNG system!
        test_rnd_generic::<f32>(0f32, 1f32, vec![-f32::INFINITY, -0f32, 0f32, f32::INFINITY]);
    }

    #[test]
    fn test_empty_f32() {
        test_empty_generic::<f32>();
    }

    #[test]
    fn test_single_f32() {
        test_single_generic::<f32>(3f32);
    }
}

mod sub_f64 {
    use super::*;

    use std::f64;

    #[test]
    fn test_rnd_f64() {
        // DO NOT use MIN/MAX here, since that overflows the RNG system!
        test_rnd_generic::<f64>(0f64, 1f64, vec![-f64::INFINITY, -0f64, 0f64, f64::INFINITY]);
    }

    #[test]
    fn test_empty_f64() {
        test_empty_generic::<f64>();
    }

    #[test]
    fn test_single_f64() {
        test_single_generic::<f64>(3f64);
    }
}
