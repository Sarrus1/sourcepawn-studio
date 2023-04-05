use logos::Logos;
use sourcepawn_lexer::Token;

fn main() {
    let input = r#"
        //#include "a"
        "int\
        hello"
        #define FOO(%1) 42%1
        stock foo() {
            int a = 2 + 2 * 2;
            string b \
            = "hello";
            return a;
        }
    "#;

    let mut lexer = Token::lexer(input);

    while let Some(token) = lexer.next() {
        println!("{:?}: {:?}", token, lexer.slice());
    }
}
