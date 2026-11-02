use std::collections::HashSet;

use xtask::source_rules::{
    forbidden_comment_lines, function_names, is_badly_named_test, is_forbidden_module_name,
    test_names,
};

fn crate_functions(names: &[&str]) -> HashSet<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[test]
fn plain_line_comment_is_reported_with_its_line() {
    assert_eq!(
        forbidden_comment_lines("fn a() {}\n// why\nfn b() {}\n"),
        [2]
    );
}

#[test]
fn block_comment_is_reported() {
    assert_eq!(forbidden_comment_lines("/* why */ fn a() {}\n"), [1]);
}

#[test]
fn doc_comments_and_safety_notes_are_accepted() {
    let source = "//! crate\n/// item\nfn a() {\n    // SAFETY: invariant\n}\n";
    assert_eq!(forbidden_comment_lines(source), Vec::<usize>::new());
}

#[test]
fn four_slashes_do_not_pass_as_documentation() {
    assert_eq!(forbidden_comment_lines("//// banner\n"), [1]);
}

#[test]
fn slashes_inside_a_string_are_not_a_comment() {
    let source = "const URL: &str = \"https://misis.ru\";\nconst RAW: &str = r#\"// x\"#;\n";
    assert_eq!(forbidden_comment_lines(source), Vec::<usize>::new());
}

#[test]
fn dumping_ground_module_names_are_forbidden() {
    assert!(is_forbidden_module_name("utils"));
    assert!(!is_forbidden_module_name("ttf"));
}

#[test]
fn prefix_naming_the_framework_counts_as_a_bad_name() {
    assert!(is_badly_named_test("test_link", &crate_functions(&[])));
}

#[test]
fn name_copied_from_a_function_counts_as_a_bad_name() {
    assert!(is_badly_named_test("link", &crate_functions(&["link"])));
}

#[test]
fn behaviour_name_is_accepted() {
    let functions = crate_functions(&["link"]);
    assert!(!is_badly_named_test(
        "later_departure_never_arrives_earlier",
        &functions
    ));
}

#[test]
fn tests_are_separated_from_ordinary_functions() {
    let source = "fn link() {}\n#[test]\nfn keeps_fifo() {}\n";
    let tests = test_names(source).unwrap();
    assert_eq!(tests[0].name, "keeps_fifo");
    assert_eq!(tests[0].line, 3);
    assert_eq!(function_names(source).unwrap(), crate_functions(&["link"]));
}

#[test]
fn tests_inside_proptest_macro_are_found() {
    let source = "proptest! { #[test] fn test_link(a in 0..1) { } }\n";
    assert_eq!(test_names(source).unwrap()[0].name, "test_link");
}

#[test]
fn function_with_another_attribute_is_not_a_test() {
    let source = "#[inline]\nfn link() {}\n";
    assert_eq!(test_names(source).unwrap().len(), 0);
    assert_eq!(function_names(source).unwrap(), crate_functions(&["link"]));
}

#[test]
fn methods_of_impls_and_traits_count_as_functions() {
    let source = "struct A;\nimpl A { fn link(&self) {} }\ntrait B { fn merge(&self); }\n";
    assert_eq!(
        function_names(source).unwrap(),
        crate_functions(&["link", "merge"])
    );
}
