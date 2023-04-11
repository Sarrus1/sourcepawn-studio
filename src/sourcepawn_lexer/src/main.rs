use sourcepawn_lexer::lexer::Token;

fn main() {
    let input = r#"int foo;
#pragma deprecated Do not use this.
void bar() {
    return;
}
"#;

    let lexer = sourcepawn_lexer::token_kind::SourcePawnLexer::new(input);
    for symbol in lexer {
        println!("{:?}", symbol)
    }
}
