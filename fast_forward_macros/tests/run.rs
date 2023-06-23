#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/two_lists.rs");
    t.pass("tests/ui/empty_list.rs");
    t.pass("tests/ui/one_indexed_list_filter.rs");
    t.pass("tests/ui/one_indexed_list_string.rs");

    t.compile_fail("tests/ui/fail_invalid_field.rs");
    t.compile_fail("tests/ui/fail_invalid_store.rs");
}
