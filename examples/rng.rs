// Writes a sequence of pseudo-random bytes to stdout.

use std::io::Write;
use xox_random::Rng;

fn main() {
  let mut g = Rng::from_seed(*b"autovivification");
  let mut out = std::io::stdout().lock();
  let a = &mut [0u8; 4096];

  loop {
    g.fill(a);
    if let Err(_) = out.write_all(a) { break; }
  }
}
