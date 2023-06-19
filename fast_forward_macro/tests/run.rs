#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/func_macro_test.rs");
}
