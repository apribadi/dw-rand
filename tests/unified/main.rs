use expect_test::expect;
use xox_random::Rng;

#[test]
fn test() {
  let mut g = Rng::new([117u8; 16]);
  expect!["0xaa14111213e90a24"].assert_eq(&format!("{:#x}", g.next()));
  expect!["0xa76761fd02073e41"].assert_eq(&format!("{:#x}", g.next()));
  expect!["0x55bd88c08f434382"].assert_eq(&format!("{:#x}", g.next()));
  expect!["0x56ec0708a27b1d26"].assert_eq(&format!("{:#x}", g.next()));
}
