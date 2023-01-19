use std::time::Instant;
use std::hint;
use xox_random::Rng as Xox;

const COUNT: usize = 100_000_000;

pub trait BenchRng {
  fn from_seed(seed: [u8; 16]) -> Self;

  fn u64(&mut self) -> u64;

  #[inline(never)]
  fn u64_noinline(&mut self) -> u64 { self.u64() }
}

impl BenchRng for Xox {
  fn from_seed(seed: [u8; 16]) -> Self {
    Self::from_seed(seed)
  }

  #[inline]
  fn u64(&mut self) -> u64 {
    self.next()
  }
}

struct Xoroshiro128pp {
  x: u64,
  y: u64,
}

impl BenchRng for Xoroshiro128pp {
  fn from_seed(seed: [u8; 16]) -> Self {
    let seed = u128::from_le_bytes(seed);
    let seed = seed | 1;
    Self { x: seed as u64, y: (seed >> 64) as u64 }
  }

  #[inline]
  fn u64(&mut self) -> u64 {
    let x = self.x;
    let y = self.y;
    let z = x.wrapping_add(y).rotate_left(17).wrapping_add(x);
    let y = x ^ y;
    let x = x.rotate_left(49) ^ y ^ y << 21;
    let y = y.rotate_left(28);
    self.x = x;
    self.y = y;
    z
  }
}

struct Pcg64dxsm {
  x: u128,
}

impl BenchRng for Pcg64dxsm {
  fn from_seed(seed: [u8; 16]) -> Self {
    Self { x: u128::from_le_bytes(seed) }
  }

  #[inline]
  fn u64(&mut self) -> u64 {
    let x = self.x;
    let a = x as u64;
    let b = (x >> 64) as u64;
    let a = a | 1;
    let b = b ^ b >> 32;
    let b = b * 0xda942042e4dd58b5;
    let b = b ^ b >> 48;
    let b = b * a;
    let x = 0xda942042e4dd58b5 * x + 1;
    self.x = x;
    b
  }
}

fn warmup() {
  let mut s = 1u64;
  for i in 0 .. 100_000_000 { s = s.wrapping_mul(i); }
  let _: u64 = hint::black_box(s);
}

fn timeit<A, F>(f: F) -> f64 where F: FnOnce() -> A {
  let start = Instant::now();
  let _: A = hint::black_box(f());
  let stop = Instant::now();
  stop.saturating_duration_since(start).as_nanos() as f64
}

fn run_bench<F>(name: &str, f: F) where F: Fn(usize, [u8; 16]) -> u64 {
  let elapsed = timeit(|| f(COUNT, *b"a starting seed."));
  print!("{:25} {:.3}ns per u64\n", name, elapsed / (COUNT as f64));
}

fn bench_loop<T: BenchRng>(count: usize, seed: [u8; 16]) -> u64 {
  let mut g = T::from_seed(seed);
  let mut s = 0u64;
  for _ in 0 .. count {
    s = s.wrapping_add(g.u64());
  }
  s
}

fn bench_loop_noinline<T: BenchRng>(count: usize, seed: [u8; 16]) -> u64 {
  let mut g = T::from_seed(seed);
  let mut s = 0u64;
  for _ in 0 .. count {
    s = s.wrapping_add(g.u64_noinline());
  }
  s
}

#[inline(never)]
fn bench_loop_pcg64dxsm(count: usize, seed: [u8; 16]) -> u64 {
  bench_loop::<Pcg64dxsm>(count, seed)
}

#[inline(never)]
fn bench_loop_xoroshiro128pp(count: usize, seed: [u8; 16]) -> u64 {
  bench_loop::<Xoroshiro128pp>(count, seed)
}

#[inline(never)]
fn bench_loop_xox(count: usize, seed: [u8; 16]) -> u64 {
  bench_loop::<Xox>(count, seed)
}

#[inline(never)]
fn bench_loop_noinline_pcg64dxsm(count: usize, seed: [u8; 16]) -> u64 {
  bench_loop_noinline::<Pcg64dxsm>(count, seed)
}

#[inline(never)]
fn bench_loop_noinline_xoroshiro128pp(count: usize, seed: [u8; 16]) -> u64 {
  bench_loop_noinline::<Xoroshiro128pp>(count, seed)
}

#[inline(never)]
fn bench_loop_noinline_xox(count: usize, seed: [u8; 16]) -> u64 {
  bench_loop_noinline::<Xox>(count, seed)
}

fn main() {
  warmup();
  run_bench("pcg64dxsm", bench_loop_pcg64dxsm);
  run_bench("xoroshiro128++", bench_loop_xoroshiro128pp);
  run_bench("xox", bench_loop_xox);
  run_bench("pcg64dxsm (noinline)", bench_loop_noinline_pcg64dxsm);
  run_bench("xoroshiro128++ (noinline)", bench_loop_noinline_xoroshiro128pp);
  run_bench("xox (noinline)", bench_loop_noinline_xox);
}
