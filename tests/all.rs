pub use xox_random::Rng;
pub use expect_test::expect;

#[test]
fn test_vectors() {
  let mut g = Rng::from_seed(*b"autovivification");
  expect!["0xe6cb90eb01058266"].assert_eq(&format!("{:#018x}", g.u64()));
  expect!["0x2d8ce79534418c6f"].assert_eq(&format!("{:#018x}", g.u64()));
  expect!["0x619cdce6110c5a54"].assert_eq(&format!("{:#018x}", g.u64()));
  expect!["0xaca4ba9f7d51d010"].assert_eq(&format!("{:#018x}", g.u64()));

  let mut g = g.split();
  expect!["0x024a1e4ef47f4d49"].assert_eq(&format!("{:#018x}", g.u64()));
  expect!["0x5b40830b87734777"].assert_eq(&format!("{:#018x}", g.u64()));
  expect!["0xa8f55647e9fbe2d3"].assert_eq(&format!("{:#018x}", g.u64()));
  expect!["0xe088780fee8e67e6"].assert_eq(&format!("{:#018x}", g.u64()));
}
