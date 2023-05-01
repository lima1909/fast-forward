// TryBuild run ui: user's interaction tests.
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui_first.rs");
    // t.pass("tests/fltrs/test_filterable_lifetime.rs");
    // t.pass("tests/fltrs/test_filterable_lifetime_many.rs");
    // t.pass("tests/fltrs/test_filterable_ignore_fields.rs");

    // t.compile_fail("tests/fltrs/test_filterable_fail.rs");
}
