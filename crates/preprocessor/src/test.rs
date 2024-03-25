use fxhash::{FxHashMap, FxHashSet};
use sourcepawn_lexer::{SourcepawnLexer, TokenKind};

use crate::evaluator::IfCondition;

fn evaluate_if_condition(input: &str) -> bool {
    let mut lexer = SourcepawnLexer::new(input);
    let mut macros = FxHashMap::default();
    let mut offsets = FxHashMap::default();
    let mut disabled_macros = FxHashSet::default();
    let mut if_condition = IfCondition::new(&mut macros, 0, &mut offsets, &mut disabled_macros);
    if let Some(symbol) = lexer.next() {
        if TokenKind::PreprocDir(sourcepawn_lexer::PreprocDir::MIf) == symbol.token_kind {
            while lexer.in_preprocessor() {
                if let Some(symbol) = lexer.next() {
                    if_condition.tokens.push(symbol.into());
                } else {
                    break;
                }
            }
        }
    }

    if_condition.evaluate().unwrap_or(false)
}

#[test]
fn if_directive_simple_true() {
    let input = r#"#if 1"#;

    assert!(evaluate_if_condition(input));
}

#[test]
fn if_directive_simple_false() {
    let input = r#"#if 0"#;

    assert!(!evaluate_if_condition(input));
}

#[test]
fn if_directive_simple_true_with_ws() {
    let input = r#"#if 1 "#;

    assert!(evaluate_if_condition(input));
}

#[test]
fn if_directive_simple_true_parenthesis() {
    let input = r#"#if (1)"#;

    assert!(evaluate_if_condition(input));
}

#[test]
fn if_directive_simple_binary_true() {
    let input = r#"#if 1+1"#;

    assert!(evaluate_if_condition(input));
}

#[test]
fn if_directive_simple_binary_false() {
    let input = r#"#if 1-1"#;

    assert!(!evaluate_if_condition(input));
}

#[test]
fn if_directive_simple_unary_false() {
    let input = r#"#if !1"#;

    assert!(!evaluate_if_condition(input));
}

#[test]
fn if_directive_equality_true() {
    let input = r#"#if 1 == 1"#;

    assert!(evaluate_if_condition(input));
}

#[test]
fn if_directive_difference_true() {
    let input = r#"#if 1 != 0"#;

    assert!(evaluate_if_condition(input));
}

#[test]
fn if_directive_equality_false() {
    let input = r#"#if 1 == 0"#;

    assert!(!evaluate_if_condition(input));
}

#[test]
fn if_directive_difference_false() {
    let input = r#"#if 1 != 1"#;

    assert!(!evaluate_if_condition(input));
}

#[test]
fn if_directive_complex_expression_1() {
    let input = r#"#if (1 + 1) && (0 + 0)"#;

    assert!(!evaluate_if_condition(input));
}

#[test]
fn if_directive_complex_expression_2() {
    let input = r#"#if (true && 1) || (true + 1)"#;

    assert!(evaluate_if_condition(input));
}

#[test]
fn if_directive_int_1() {
    let input = r#"#if 1000 == 1000"#;

    assert!(evaluate_if_condition(input));
}

#[test]
fn if_directive_int_2() {
    let input = r#"#if (10_00) == 1000"#;

    assert!(evaluate_if_condition(input));
}

#[test]
fn if_directive_binary_1() {
    let input = r#"#if 0x0001 == 0x0001"#;

    assert!(evaluate_if_condition(input));
}

#[test]
fn if_directive_binary_2() {
    let input = r#"#if 0x000_1 == 0x000_1"#;

    assert!(evaluate_if_condition(input));
}

#[test]
fn if_directive_char_1() {
    let input = r#"#if 'a' == 'a'"#;

    assert!(evaluate_if_condition(input));
}

#[test]
fn if_directive_char_2() {
    let input = r#"#if 'a' != 'b'"#;

    assert!(evaluate_if_condition(input));
}
