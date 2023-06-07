// TryBuild run ui: user's interaction tests.
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui_first.rs");

    t.compile_fail("tests/ui/fail_no_index_set.rs");
    t.compile_fail("tests/ui/fail_unnamed_struct.rs");
}
