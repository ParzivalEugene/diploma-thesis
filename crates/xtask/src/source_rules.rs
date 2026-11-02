use std::collections::HashSet;

use proc_macro2::{TokenStream, TokenTree};
use ra_ap_rustc_lexer::{FrontmatterAllowed, TokenKind, tokenize};
use syn::visit::Visit;

const FORBIDDEN_MODULE_NAMES: [&str; 5] = ["utils", "helpers", "common", "misc", "shared"];
const ALLOWED_COMMENT_PREFIXES: [&str; 3] = ["// SAFETY:", "//!", "///"];

pub const LARGE_FILE_LINES: usize = 400;

#[derive(Debug, PartialEq, Eq)]
pub struct NamedLine {
    pub name: String,
    pub line: usize,
}

pub fn is_forbidden_module_name(file_stem: &str) -> bool {
    FORBIDDEN_MODULE_NAMES.contains(&file_stem)
}

pub fn forbidden_comment_lines(source: &str) -> Vec<usize> {
    let mut offset = 0;
    let mut lines = Vec::new();
    for token in tokenize(source, FrontmatterAllowed::No) {
        let end = offset + token.len as usize;
        let text = &source[offset..end];
        let is_comment = matches!(
            token.kind,
            TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }
        );
        if is_comment && !is_allowed_comment(text) {
            lines.push(1 + source[..offset].matches('\n').count());
        }
        offset = end;
    }
    lines
}

fn is_allowed_comment(text: &str) -> bool {
    !text.starts_with("////")
        && ALLOWED_COMMENT_PREFIXES
            .iter()
            .any(|prefix| text.starts_with(prefix))
}

pub fn function_names(source: &str) -> syn::Result<HashSet<String>> {
    let file = syn::parse_file(source)?;
    let mut collector = FunctionCollector::default();
    collector.visit_file(&file);
    Ok(collector.names)
}

pub fn test_names(source: &str) -> syn::Result<Vec<NamedLine>> {
    let file = syn::parse_file(source)?;
    let mut collector = FunctionCollector::default();
    collector.visit_file(&file);
    Ok(collector.tests)
}

pub fn is_badly_named_test(test_name: &str, crate_functions: &HashSet<String>) -> bool {
    test_name.starts_with("test_") || crate_functions.contains(test_name)
}

#[derive(Default)]
struct FunctionCollector {
    names: HashSet<String>,
    tests: Vec<NamedLine>,
}

impl FunctionCollector {
    fn record(&mut self, attributes: &[syn::Attribute], ident: &syn::Ident) {
        if attributes.iter().any(is_test_attribute) {
            self.tests.push(NamedLine {
                name: ident.to_string(),
                line: ident.span().start().line,
            });
        } else {
            self.names.insert(ident.to_string());
        }
    }

    fn record_proptest_body(&mut self, tokens: TokenStream) {
        let mut stream = tokens.into_iter().peekable();
        while let Some(token) = stream.next() {
            if let TokenTree::Ident(keyword) = &token
                && keyword == "fn"
                && let Some(TokenTree::Ident(name)) = stream.peek()
            {
                self.tests.push(NamedLine {
                    name: name.to_string(),
                    line: name.span().start().line,
                });
            }
        }
    }
}

fn is_test_attribute(attribute: &syn::Attribute) -> bool {
    attribute
        .path()
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "test")
}

impl<'ast> Visit<'ast> for FunctionCollector {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.record(&item.attrs, &item.sig.ident);
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.record(&item.attrs, &item.sig.ident);
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        self.record(&item.attrs, &item.sig.ident);
        syn::visit::visit_trait_item_fn(self, item);
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        if item
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "proptest")
        {
            self.record_proptest_body(item.tokens.clone());
        }
        syn::visit::visit_macro(self, item);
    }
}
