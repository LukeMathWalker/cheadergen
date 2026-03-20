#[test]
fn config_attribute() {
    let t = trybuild::TestCases::new();
    t.pass("tests/config/success/*.rs");
    t.compile_fail("tests/config/fail/*.rs");
}
