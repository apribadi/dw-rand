use std::time::Instant;
use std::hint;
use xox_random::Rng as Xox;
use trivium::Trivium64;

const COUNT: usize = 100_000_000;

#[inline(always)]
const fn umulh(x: u64, y: u64) -> u64 {
  (((x as u128) * (y as u128)) >> 64) as u64
}

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

impl BenchRng for Trivium64 {
  fn from_seed(seed: [u8; 16]) -> Self {
    let key = seed[.. 10].try_into().unwrap();
    let iv = [ &seed[10 .. 16], &[0u8; 4] ].concat().try_into().unwrap();
    Self::new(key, iv)
  }

  #[inline]
  fn u64(&mut self) -> u64 {
    self.next()
  }
}

struct Mwc256xxa64 {
  x: u64,
  y: u64,
  z: u64,
  c: u64,
}

impl BenchRng for Mwc256xxa64 {
  fn from_seed(seed: [u8; 16]) -> Self {
    let seed = u128::from_le_bytes(seed);
    let x = seed as u64;
    let y = (seed >> 64) as u64;
    Self {
      x,
      y,
      z: 0xcafef00dd15ea5e5,
      c: 0x14057b7ef767814f,
    }
  }

  fn u64(&mut self) -> u64 {
    const M: u64 = 0xfeb3_4465_7c0a_f413;
    let x = self.x;
    let y = self.y;
    let z = self.z;
    let c = self.c;
    let lo = z.wrapping_mul(M);
    let hi = umulh(z, M);
    let (w, p) = lo.overflowing_add(c);
    self.x = w;
    self.y = x;
    self.z = y;
    self.c = hi.wrapping_add(p as u64);
    (y ^ z).wrapping_add(x ^ hi)
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

fn run_bench<T: BenchRng, F>(name: &str, f: F) where F: Fn(&mut T, usize) -> u64 {
  let mut rng = T::from_seed(*b"a starting seed.");
  let elapsed = timeit(|| f(&mut rng, COUNT));
  // let rate = ((COUNT as f64) * 1_000.) / elapsed;
  // print!("{:25} {:.3} / μs\n", name, rate);
  print!("{:25} {:.3} ns\n", name, elapsed / (COUNT as f64));
}

fn bench_loop<T: BenchRng>(rng: &mut T, count: usize) -> u64 {
  let mut s = 0u64;
  for _ in 0 .. count {
    s = s.wrapping_add(rng.u64());
  }
  s
}

#[inline(never)]
fn bench_loop_2x(rng: &mut Xox, count: usize) -> u64 {
  let mut rng2 = rng.split();
  let mut s = 0u64;
  for _ in 0 .. count / 2 {
    s = s.wrapping_add(rng.u64());
    s = s.wrapping_add(rng2.u64());
  }
  s
}

fn bench_loop_noinline<T: BenchRng>(rng: &mut T, count: usize) -> u64 {
  let mut s = 0u64;
  for _ in 0 .. count {
    s = s.wrapping_add(rng.u64_noinline());
  }
  s
}

#[inline(never)]
fn bench_loop_pcg64dxsm(rng: &mut Pcg64dxsm, count: usize) -> u64 {
  bench_loop::<Pcg64dxsm>(rng, count)
}

#[inline(never)]
fn bench_loop_xoroshiro128pp(rng: &mut Xoroshiro128pp, count: usize) -> u64 {
  bench_loop::<Xoroshiro128pp>(rng, count)
}

#[inline(never)]
fn bench_loop_xox(rng: &mut Xox, count: usize) -> u64 {
  bench_loop::<Xox>(rng, count)
}

#[inline(never)]
fn bench_loop_trivium(rng: &mut Trivium64, count: usize) -> u64 {
  bench_loop::<Trivium64>(rng, count)
}

#[inline(never)]
fn bench_loop_mwc256xxa64(rng: &mut Mwc256xxa64, count: usize) -> u64 {
  bench_loop::<Mwc256xxa64>(rng, count)
}

#[inline(never)]
fn bench_loop_noinline_pcg64dxsm(rng: &mut Pcg64dxsm, count: usize) -> u64 {
  bench_loop_noinline::<Pcg64dxsm>(rng, count)
}

#[inline(never)]
fn bench_loop_noinline_xoroshiro128pp(rng: &mut Xoroshiro128pp, count: usize) -> u64 {
  bench_loop_noinline::<Xoroshiro128pp>(rng, count)
}

#[inline(never)]
fn bench_loop_noinline_xox(rng: &mut Xox, count: usize) -> u64 {
  bench_loop_noinline::<Xox>(rng, count)
}

#[inline(never)]
fn bench_loop_noinline_trivium(rng: &mut Trivium64, count: usize) -> u64 {
  bench_loop_noinline::<Trivium64>(rng, count)
}

#[inline(never)]
fn bench_loop_noinline_mwc256xxa64(rng: &mut Mwc256xxa64, count: usize) -> u64 {
  bench_loop_noinline::<Mwc256xxa64>(rng, count)
}

fn main() {
  warmup();
  run_bench("pcg64dxsm", bench_loop_pcg64dxsm);
  run_bench("xoroshiro128++", bench_loop_xoroshiro128pp);
  run_bench("xox", bench_loop_xox);
  run_bench("xox 2x", bench_loop_2x);
  run_bench("trivium", bench_loop_trivium);
  run_bench("mwc256xxa64", bench_loop_mwc256xxa64);
  run_bench("pcg64dxsm (noinline)", bench_loop_noinline_pcg64dxsm);
  run_bench("xoroshiro128++ (noinline)", bench_loop_noinline_xoroshiro128pp);
  run_bench("xox (noinline)", bench_loop_noinline_xox);
  run_bench("trivium (noinline)", bench_loop_noinline_trivium);
  run_bench("mwc256xxa64 (noinline)", bench_loop_noinline_mwc256xxa64);
}
