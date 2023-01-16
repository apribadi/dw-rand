use core::array;
use core::cell::Cell;
use core::hint;

#[derive(Clone)]
pub struct Rng {
  x: u64,
  y: u64,
}

#[inline(always)]
const fn umulh(x: u64, y: u64) -> u64 {
  (((x as u128) * (y as u128)) >> 64) as u64
}

impl Rng {
  #[inline]
  pub const fn new(seed: [u8; 16]) -> Self {
    let s = u128::from_le_bytes(seed);
    let x = (s as u64) | 1;
    let y = (s >> 64) as u64;
    Self { x, y }
  }

  #[inline(never)]
  #[cold]
  pub fn from_system_seed() -> Self {
    let mut seed = [0; 16];
    getrandom::getrandom(&mut seed).expect("getrandom::getrandom failed!");
    Self::new(seed)
  }

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

  #[inline]
  pub fn gen<const N: usize>(&mut self) -> [u64; N] {
    array::from_fn(|_| self.next())
  }

  #[inline]
  pub fn split(&mut self) -> Self {
    Self { x: self.next() | 1, y: self.next() }
  }

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
  use super::*;

  std::thread_local! {
    static RNG: Cell<[u64; 2]> = const { Cell::new([ 0, 0 ]) };
  }

  #[inline(always)]
  fn with_non_reentrant<F, A>(f: F) -> A where F: FnOnce(&mut Rng) -> A {
    RNG.with(|t| {
      let [ x, y ] = t.get();
      let mut g = if x == 0 && y == 0 { Rng::from_system_seed() } else { Rng { x, y } };
      let a = f(&mut g);
      t.set([ g.x, g.y ]);
      a
    })
  }

  #[inline]
  pub fn gen<const N: usize>() -> [u64; N] {
    with_non_reentrant(Rng::gen)
  }

  #[inline]
  pub fn split() -> Rng {
    with_non_reentrant(Rng::split)
  }
}
