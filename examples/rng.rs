use core::num::NonZeroU128;
use std::io::Write;
use xox_random::Rng;

fn main() {
  let mut rng = Rng::new([117u8; 16]);
  let mut buf = vec![0u8; 8192].into_boxed_slice();
  let mut out = std::io::stdout().lock();

  loop {
    rng.fill(&mut buf);
    out.write_all(&buf).expect("write_all failed!");
  }
}
