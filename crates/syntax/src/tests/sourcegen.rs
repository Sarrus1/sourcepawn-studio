//! This module generates the [`TSKind`](crate::TSKind) enum and its methods
//! from the tree-sitter grammar.
//!
//! It allows for easier pattern matching against node kinds and avoids
//! potential mistakes when the tree-sitter grammar is updated.
//! Comparing ints is also much faster than computing strings.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use test_utils::{ensure_file_contents, project_root};
use tree_sitter_sourcepawn::language;
use xshell::{cmd, Shell};

#[test]
/// Generate the [`TSKind`](crate::TSKind) enum.
fn generate_node_kinds() {
    let node_kind_count = language().node_kind_count();
    let mut colon_count = 0;
    let entries = (0..node_kind_count).map(|kind_id| {
        let name = language().node_kind_for_id(kind_id as u16).unwrap();
        let sanitized = sanitize_identifier(name);
        // This is a hack to avoid deduplicating the anonymous ":" in the tree-sitter grammar.
        // It works...
        let name = if language().node_kind_is_named(kind_id as u16) {
            match syn::parse_str::<syn::Ident>(&sanitized).ok() {
                Some(name) => name,
                None => format_ident!("r#{}", sanitized),
            }
        } else {
            if sanitized == "COLON" {
                colon_count += 1;
            }
            if colon_count == 2 {
                format_ident!("anon_{}_", sanitized)
            } else {
                format_ident!("anon_{}", sanitized)
            }
        };
        let kind_id: TokenStream = format!("{}", kind_id).parse().unwrap();
        quote! { #name = #kind_id }
    });
    let stream = quote! {
        #![allow(bad_style, missing_docs, unreachable_pub, unused)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        #[repr(u16)]
        pub enum TSKind {
            #(#entries),*
        }

        impl From<tree_sitter::Node<'_>> for TSKind {
            fn from(v: tree_sitter::Node<'_>) -> Self {
                unsafe { ::std::mem::transmute(v.kind_id()) }
            }
        }

        impl From<&tree_sitter::Node<'_>> for TSKind {
            fn from(v: &tree_sitter::Node<'_>) -> Self {
                Self::from(*v)
            }
        }
    };
    let path = project_root().join("crates/syntax/src/generated.rs");
    ensure_file_contents(&path, &reformat(stream.to_string()));
}

/// Exact copy of the identifier sanitization logic of tree-sitter.
/// https://github.com/tree-sitter/tree-sitter/blob/660481dbf71413eba5a928b0b0ab8da50c1109e0/cli/src/generate/render.rs#L1536-L1629
fn sanitize_identifier(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            result.push(c);
        } else {
            'special_chars: {
                let replacement = match c {
                    ' ' if name.len() == 1 => "SPACE",
                    '~' => "TILDE",
                    '`' => "BQUOTE",
                    '!' => "BANG",
                    '@' => "AT",
                    '#' => "POUND",
                    '$' => "DOLLAR",
                    '%' => "PERCENT",
                    '^' => "CARET",
                    '&' => "AMP",
                    '*' => "STAR",
                    '(' => "LPAREN",
                    ')' => "RPAREN",
                    '-' => "DASH",
                    '+' => "PLUS",
                    '=' => "EQ",
                    '{' => "LBRACE",
                    '}' => "RBRACE",
                    '[' => "LBRACK",
                    ']' => "RBRACK",
                    '\\' => "BSLASH",
                    '|' => "PIPE",
                    ':' => "COLON",
                    ';' => "SEMI",
                    '"' => "DQUOTE",
                    '\'' => "SQUOTE",
                    '<' => "LT",
                    '>' => "GT",
                    ',' => "COMMA",
                    '.' => "DOT",
                    '?' => "QMARK",
                    '/' => "SLASH",
                    '\n' => "LF",
                    '\r' => "CR",
                    '\t' => "TAB",
                    '\0' => "NULL",
                    '\u{0001}' => "SOH",
                    '\u{0002}' => "STX",
                    '\u{0003}' => "ETX",
                    '\u{0004}' => "EOT",
                    '\u{0005}' => "ENQ",
                    '\u{0006}' => "ACK",
                    '\u{0007}' => "BEL",
                    '\u{0008}' => "BS",
                    '\u{000b}' => "VTAB",
                    '\u{000c}' => "FF",
                    '\u{000e}' => "SO",
                    '\u{000f}' => "SI",
                    '\u{0010}' => "DLE",
                    '\u{0011}' => "DC1",
                    '\u{0012}' => "DC2",
                    '\u{0013}' => "DC3",
                    '\u{0014}' => "DC4",
                    '\u{0015}' => "NAK",
                    '\u{0016}' => "SYN",
                    '\u{0017}' => "ETB",
                    '\u{0018}' => "CAN",
                    '\u{0019}' => "EM",
                    '\u{001a}' => "SUB",
                    '\u{001b}' => "ESC",
                    '\u{001c}' => "FS",
                    '\u{001d}' => "GS",
                    '\u{001e}' => "RS",
                    '\u{001f}' => "US",
                    '\u{007F}' => "DEL",
                    '\u{FEFF}' => "BOM",
                    '\u{0080}'..='\u{FFFF}' => {
                        result.push_str(&format!("u{:04x}", c as u32));
                        break 'special_chars;
                    }
                    '\u{10000}'..='\u{10FFFF}' => {
                        result.push_str(&format!("U{:08x}", c as u32));
                        break 'special_chars;
                    }
                    '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' => unreachable!(),
                    ' ' => break 'special_chars,
                };
                if !result.is_empty() && !result.ends_with('_') {
                    result.push('_');
                }
                result += replacement;
            }
        }
    }
    result
}

/// Ensure rustfmt is installed.
fn ensure_rustfmt(sh: &Shell) {
    let version = cmd!(sh, "rustup run stable rustfmt --version")
        .read()
        .unwrap_or_default();
    if !version.contains("stable") {
        panic!(
            "Failed to run rustfmt from toolchain 'stable'. \
                 Please run `rustup component add rustfmt --toolchain stable` to install it.",
        );
    }
}

/// Format valid Rust code with rustfmt.
fn reformat(text: String) -> String {
    let sh = Shell::new().unwrap();
    ensure_rustfmt(&sh);
    let mut stdout = cmd!(sh, "rustup run stable rustfmt --config fn_single_line=true")
        .stdin(text)
        .read()
        .unwrap();
    if !stdout.ends_with('\n') {
        stdout.push('\n');
    }
    stdout
}
