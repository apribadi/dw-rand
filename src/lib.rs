#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#![deny(unsafe_op_in_unsafe_fn)]
#![warn(elided_lifetimes_in_paths)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused_lifetimes)]
#![warn(unused_qualifications)]
#![warn(unused_results)]

use core::array;
use core::num::NonZeroU128;

/// A fast non-cryptographic random number generator.

#[derive(Clone)]
pub struct Rng(NonZeroU128);

#[inline(always)]
const fn umulh(x: u64, y: u64) -> u64 {
  (((x as u128) * (y as u128)) >> 64) as u64
}

impl Rng {
  /// Creates a new random number generator starting from the given state. A
  /// good start state should be drawn from a distribution with sufficient
  /// entropy.

  #[inline(always)]
  pub const fn new(state: NonZeroU128) -> Self {
    Self(state)
  }

  /// Creates a new random number generator using the given seed to create the
  /// start state. A good seed should be drawn from a distribution with
  /// sufficient entropy.

  #[inline(always)]
  pub const fn from_seed(seed: [u8; 16]) -> Self {
    let s = u128::from_le_bytes(seed);
    let s = s | 1;
    let s = unsafe { NonZeroU128::new_unchecked(s) };
    Self::new(s)
  }

  /// Creates a new random number generator with a seed requested from the
  /// system through a method that may depend on the platform.

  #[cfg(feature = "getrandom")]
  #[inline(never)]
  #[cold]
  pub fn from_system_seed() -> Self {
    let mut seed = [0; 16];
    getrandom::getrandom(&mut seed).expect("getrandom::getrandom failed!");
    Self::from_seed(seed)
  }

  /// Accesses the random number generator's current state.

  #[inline(always)]
  pub const fn state(&self) -> NonZeroU128 {
    self.0
  }

  /// Splits off a new random number generator that may be used in addition to
  /// the original.

  #[inline(always)]
  pub fn split(&mut self) -> Self {
    let x = self.u64();
    let y = self.u64();
    let s = (x as u128) | ((y as u128) << 64);
    let s = s | 1;
    let s = unsafe { NonZeroU128::new_unchecked(s) };
    Self::new(s)
  }

  /// Samples a `u64` from the uniform distribution.

  #[inline(always)]
  pub fn u64(&mut self) -> u64 {
    let s = self.0;
    let s = s.get();
    let a = s as u64;
    let b = (s >> 64) as u64;
    let c = a.rotate_right(7) ^ b;
    let d = a ^ a >> 19;
    let x = c.wrapping_add(a.wrapping_mul(b) ^ umulh(a, b));
    let s = (c as u128) | ((d as u128) << 64);
    let s = unsafe { NonZeroU128::new_unchecked(s) };
    self.0 = s;
    x
  }

  /// Samples an array of i.i.d. `u64`s from the uniform distribution.

  #[inline(always)]
  pub fn array_u64<const N: usize>(&mut self ) -> [u64; N] {
    array::from_fn(|_| self.u64())
  }

  /// Fills a slice with a i.i.d. bytes sampled from the uniform distribution.

  pub fn fill(&mut self, mut dst: &mut [u8]) {
    if dst.len() == 0 { return; }

    let mut x;

    loop {
      x = self.u64();
      if dst.len() < 8 { break; }
      dst[.. 8].copy_from_slice(&x.to_le_bytes());
      dst = &mut dst[8 ..];
    }

    while dst.len() > 0 {
      dst[0] = x as u8;
      x >>= 8;
      dst = &mut dst[1 ..];
    }
  }
}

#[cfg(feature = "thread-local")]
pub mod thread_local {
  //! This module provides access to a thread-local instance of the random
  //! number generator.

  use super::*;
  use core::cell::Cell;

  std::thread_local! {
    static RNG: Cell<u128> = const { Cell::new(0) };
  }

  #[inline(always)]
  fn with<F, A>(f: F) -> A
  where
    F: FnOnce(&mut Rng) -> A
  {
    RNG.with(|c| {
      let s = c.get();
      let s = NonZeroU128::new(s);
      let g = match s { None => Rng::from_system_seed(), Some(s) => Rng::new(s) };
      let mut g = g;
      let x = f(&mut g);
      let s = g.state();
      let s = s.get();
      c.set(s);
      x
    })
  }

  /// Splits off a random number generator from the thread-local instance.
  ///
  /// If you need to generate many random numbers, then it is good to first get
  /// a `split` generator because you can then use that generator without
  /// accessing thread-local storage.

  pub fn split() -> Rng {
    with(Rng::split)
  }

  /// Samples a `u64` from the uniform distribution.

  pub fn u64() -> u64 {
    with(Rng::u64)
  }

  /// Samples an array of i.i.d. `u64`s from the uniform distribution.

  pub fn array_u64<const N: usize>() -> [u64; N] {
    with(Rng::array_u64)
  }

  /// Fills a slice with a i.i.d. bytes sampled from the uniform distribution.

  pub fn fill(dst: &mut [u8]) {
    with(|g| g.fill(dst))
  }
}
