use expect_test::expect;
use xox_random::Rng;

#[test]
fn test() {
  let mut g = Rng::new(*b"autovivification");
  expect!["0xe6cb90eb01058266"].assert_eq(&format!("{:#x}", g.next()));
  expect!["0x2d8ce79534418c6f"].assert_eq(&format!("{:#x}", g.next()));
  expect!["0x619cdce6110c5a54"].assert_eq(&format!("{:#x}", g.next()));
  expect!["0xaca4ba9f7d51d010"].assert_eq(&format!("{:#x}", g.next()));
}
