//! This module implements a fast non-cryptographic random number generator.
//!
//! The random number generator has a state space of size `2**128 - 1` and also
//! a period of length `2**128 - 1`. I believe that it generates high-quality
//! random numbers, but should not be used for cryptographic purposes. It is
//! very fast.
//!
//! TODO: practrand results
//!
//! TODO: benchmarks
//!
//! # Design
//!
//! ## The State Space and Output Space
//!
//! Like many other designs for random number generators, this design can be
//! thought of as a combination of two components: a state transition function
//! and an output function. Let `U` be the state space and `V` be the output
//! space. Then with the two functions
//!
//! ```text
//! f : U -> U
//! g : U -> V
//! ```
//!
//! the `i`th state and output are
//!
//! ```text
//! u_i = f(f(... f(u_0)))
//!       \_______/
//!        i times
//!
//! v_i = g(u_i)
//! ```
//!
//! respectively. In our case, the state space is `NonZeroU128` and the
//! output space is `u64`.
//!
//! ```text
//! f : NonZeroU128 -> NonZeroU128
//! g : NonZeroU128 -> u64
//! ```
//!
//! The size of the state space was chosen because
//! 64 bits is too small for some plausible applications, while 128 bits should
//! be sufficient for almost all non-cryptographic purposes.
//!
//! ## The State Transition Function and its Period
//!
//! The state transition function is a member of `GL(128, 2)`, that is, it is
//! an invertible linear transformation from the vector space of dimension 128
//! over the finite field of order 2 to itself.
//!
//! In order to see that `f` is invertible, note that ...
//!
//! TODO
//!
//! Checking that `f` has period `2**128 - 1` takes a bit of computation. Let
//! `A` be the binary matrix corresponding to `f`. We can take `A` to the power
//! of `2**128 - 1` using `O(log(n))` exponentiation and verify that it is the
//! identity matrix.
//!
//! Also, we can factor `2**128 - 1` first into a product of Fermat numbers and
//! then into a product of primes.
//!
//! ```text
//! 2**128 - 1 = (2**1 + 1) (2**2 + 1) (2**4 + 1) (2**8 + 1) (2**16 + 1) (2**32 + 1)
//!            = 3 * 5 * 17 * 257 * 65537 * 641 * 6700417
//! ```
//!
//! Then it is sufficient to check that `A ** ((2**128 - 1) / p_i)` is *not*
//! the identity for each prime factor `p_i` and to recall some elementary
//! facts about finite groups.
//!
//! ## The Output Function
//!
//! ## A Survey of Alternate State Transition Functions
//!
//! - counter
//!
//! - LCG
//!
//! - LFSR
//!
//! - xorshift & co
//!
//! - approximating a random invertible transition
//!
//! ## A Survey of Alternate Output Functions
//!
//! - projection
//!
//! - xor, add
//!
//! - hash mixer
//!
//! ## Comparisons with Selected RNGs
//!
//! - lcg128
//!
//! - pcg64-dxsm
//!
//! - xorshift128+
//!
//! - xoroshiro128++
//!
//! - romuduo
//!
//! - splitmix64
//!
//! - wyrand
//!
//! - lxm-l64x128

use core::array;
use core::cell::Cell;
use core::hint;
use core::num::NonZeroU128;

/// A fast non-cryptographic random number generator.
///
/// The [module documentation](self) discusses the design of the generator.

#[derive(Clone)]
pub struct Rng {
  x: u64,
  y: u64,
}

#[inline(always)]
const fn umulh(x: u64, y: u64) -> u64 {
  (((x as u128) * (y as u128)) >> 64) as u64
}

#[inline(always)]
const fn make_state(seed: [u8; 16]) -> NonZeroU128 {
  match NonZeroU128::new(u128::from_le_bytes(seed)) {
    None => unsafe { NonZeroU128::new_unchecked(1) },
    Some(seed) => seed
  }
}

#[inline(never)]
#[cold]
fn get_system_seed() -> [u8; 16] {
  let mut seed = [0; 16];
  getrandom::getrandom(&mut seed).expect("getrandom::getrandom failed!");
  seed
}

impl Rng {
  /// Creates a new random number generator starting from the given state. A
  /// good start state should be drawn from a distribution with sufficient
  /// entropy.

  #[inline]
  pub const fn new(state: NonZeroU128) -> Self {
    let s = state.get();
    let x = s as u64;
    let y = (s >> 64) as u64;
    Self { x, y }
  }

  /// Creates a new random number generator using the given seed to create the
  /// start state. A good seed should be drawn from a distribution with
  /// sufficient entropy.

  #[inline]
  pub const fn from_seed(seed: [u8; 16]) -> Self {
    Self::new(make_state(seed))
  }

  /// Creates a new random number generator with a seed requested from the
  /// system through a method that may depend on the platform.

  #[inline]
  pub fn from_system_seed() -> Self {
    Self::from_seed(get_system_seed())
  }

  /// Accesses the random number generator's current state.

  #[inline]
  pub fn state(&self) -> NonZeroU128 {
    let x = self.x;
    let y = self.y;
    let s = (x as u128) | ((y as u128) << 64);
    unsafe { NonZeroU128::new_unchecked(s) }
  }

  /// Samples a `u64` from the uniform distribution.

  #[inline]
  pub fn next(&mut self) -> u64 {
    let x = self.x;
    let y = self.y;
    let a = x.rotate_right(7) ^ y;
    let b = x ^ x >> 19;
    let c = a.wrapping_add(x.wrapping_mul(y) ^ umulh(x, y));
    self.x = a;
    self.y = b;
    c
  }

  /// Samples a chunk of i.i.d. `u64`s from the uniform distribution.

  #[inline]
  pub fn chunk<const N: usize>(&mut self) -> [u64; N] {
    array::from_fn(|_| self.next())
  }

  /// Splits off a new random number generator that may be used in addition to
  /// the original.

  #[inline]
  pub fn split(&mut self) -> Self {
    Self { x: self.next() | 1, y: self.next() }
  }

  /// Fills a slice with a sample of i.i.d. bytes from the uniform
  /// distribution.

  pub fn fill(&mut self, mut dst: &mut [u8]) {
    if dst.len() == 0 { return; }

    let mut x;

    loop {
      x = self.next().to_le_bytes();

      if dst.len() <= 8 { break; }

      dst[.. 8].copy_from_slice(&x);
      dst = &mut dst[8 ..];
    }

    match dst.len() {
      1 => { dst.copy_from_slice(&x[.. 1]) }
      2 => { dst.copy_from_slice(&x[.. 2]) }
      3 => { dst.copy_from_slice(&x[.. 3]) }
      4 => { dst.copy_from_slice(&x[.. 4]) }
      5 => { dst.copy_from_slice(&x[.. 5]) }
      6 => { dst.copy_from_slice(&x[.. 6]) }
      7 => { dst.copy_from_slice(&x[.. 7]) }
      8 => { dst.copy_from_slice(&x[.. 8]) }
      _ => { unsafe { hint::unreachable_unchecked() } }
    }
  }
}

pub mod thread_local {
  //! This module provides access to a thread-local instance of the random
  //! number generator.

  use super::*;

  std::thread_local! {
    static RNG: Cell<u128> = const { Cell::new(0) };
  }

  #[inline(always)]
  fn with_rng_non_reentrant<F, A>(f: F) -> A where F: FnOnce(&mut Rng) -> A {
    RNG.with(|c| {
      let s = c.get();
      let s = NonZeroU128::new(s).unwrap_or_else(|| make_state(get_system_seed()));
      let mut g = Rng::new(s);
      let a = f(&mut g);
      c.set(u128::from(g.state()));
      a
    })
  }

  /// Samples a chunk of i.i.d. `u64`s from the uniform distribution.
  ///
  /// It is better to sample one larger chunk rather than multiple smaller
  /// chunks, because doing so reduces the number of accesses to thread-local
  /// storage.

  pub fn chunk<const N: usize>() -> [u64; N] {
    with_rng_non_reentrant(Rng::chunk)
  }

  /// Splits off a new random number generator from the thread-local generator.
  ///
  /// If you need to generate many random numbers, then it is good to first get
  /// a `split` generator because you can then use that generator without
  /// accessing thread-local storage.

  pub fn split() -> Rng {
    with_rng_non_reentrant(Rng::split)
  }
}
