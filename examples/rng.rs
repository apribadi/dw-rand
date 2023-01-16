use std::io::Write;
use xox_random::Rng;

fn main() {
  let mut g = Rng::new(*b"autovivification");
  let mut out = std::io::stdout().lock();
  let a = &mut [0u8; 4096];

  loop {
    g.fill(a);
    if let Err(_) = out.write_all(a) { break; }
  }
}
