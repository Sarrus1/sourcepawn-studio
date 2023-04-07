use logos::Logos;
use sourcepawn_lexer::Token;

fn main() {
    let input = r#"#pragma deprecated foo
"#;

    let mut lexer = Token::lexer(input);

    while let Some(token) = lexer.next() {
        println!("{:?}: {:?}", token, lexer.slice());
    }
}
