#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Fn, Give, If, Else, Elif, For, While, In, Stop, Skip,
    Type, Use, From, As, Try, Catch, Finally, Throw,
    Match, Case, Async, Wait, Yield, True, False, Nothing, Const,
    Begin, End,
    And, Or, Not,

    // Identifiers and Literals
    Ident(String),
    String(String),
    Number(f64),

    // Symbols
    Bang,       // !
    Assign,     // =
    EqEq,       // ==
    BangEq,     // !=
    Less,       // <
    Greater,    // >
    LessEq,     // <=
    GreaterEq,  // >=
    SayBang,    // say!
    AskBang,    // ask!
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    LParen,     // (
    RParen,     // )
    LBracket,   // [
    RBracket,   // ]
    LBrace,     // {
    RBrace,     // }
    Comma,      // ,
    Colon,      // :
    Dot,        // .

    // End of file
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Self { token_type, line, column }
    }
}
