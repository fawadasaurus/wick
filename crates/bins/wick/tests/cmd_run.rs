#[test]
fn cli_tests() {
  trycmd::TestCases::new().case("tests/cmd/run/*.toml");
}
