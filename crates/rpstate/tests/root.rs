#[cfg(feature = "redb")]
#[test]
fn test_expansion() {
    macrotest::expand("tests/expand/*.rs");
}
