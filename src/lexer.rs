use crate::token::{Token, TokenType};
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    source: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars().peekable(),
            line: 1,
            column: 1,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.source.next();
        if let Some(ch) = c {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        c
    }

    fn peek(&mut self) -> Option<&char> {
        self.source.peek()
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_string(&mut self) -> Result<Token, String> {
        let start_col = self.column;
        let mut s = String::new();
        // The opening quote was already consumed by next_token
        while let Some(&c) = self.peek() {
            if c == '"' {
                self.advance(); // consume closing quote
                return Ok(Token::new(TokenType::String(s), self.line, start_col));
            } else {
                s.push(c);
                self.advance();
            }
        }
        Err(format!("Unterminated string at line {}", self.line))
    }

    fn read_number(&mut self, first: char) -> Token {
        let start_col = self.column - 1;
        let mut s = String::new();
        s.push(first);

        while let Some(&c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let num: f64 = s.parse().unwrap_or(0.0); // Simple fallback for now
        Token::new(TokenType::Number(num), self.line, start_col)
    }

    fn read_identifier_or_keyword(&mut self, first: char) -> Token {
        let start_col = self.column - 1;
        let mut s = String::new();
        s.push(first);

        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '!' {
                s.push(c);
                self.advance();
                if c == '!' {
                    break; // e.g. say! ask!
                }
            } else {
                break;
            }
        }

        let tt = match s.as_str() {
            "fn" => TokenType::Fn,
            "give" => TokenType::Give,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "elif" => TokenType::Elif,
            "for" => TokenType::For,
            "while" => TokenType::While,
            "in" => TokenType::In,
            "stop" => TokenType::Stop,
            "skip" => TokenType::Skip,
            "type" => TokenType::Type,
            "use" => TokenType::Use,
            "from" => TokenType::From,
            "as" => TokenType::As,
            "try" => TokenType::Try,
            "catch" => TokenType::Catch,
            "finally" => TokenType::Finally,
            "throw" => TokenType::Throw,
            "match" => TokenType::Match,
            "case" => TokenType::Case,
            "async" => TokenType::Async,
            "wait" => TokenType::Wait,
            "yield" => TokenType::Yield,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "nothing" => TokenType::Nothing,
            "const" => TokenType::Const,
            "begin" => TokenType::Begin,
            "end" => TokenType::End,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "not" => TokenType::Not,
            "say!" => TokenType::SayBang,
            "ask!" => TokenType::AskBang,
            _ => TokenType::Ident(s),
        };

        Token::new(tt, self.line, start_col)
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();
        let col = self.column;

        if let Some(c) = self.advance() {
            match c {
                '!' => {
                    if let Some(&'=') = self.peek() {
                        self.advance();
                        Ok(Token::new(TokenType::BangEq, self.line, col))
                    } else {
                        Ok(Token::new(TokenType::Bang, self.line, col))
                    }
                }
                '=' => {
                    if let Some(&'=') = self.peek() {
                        self.advance();
                        Ok(Token::new(TokenType::EqEq, self.line, col))
                    } else {
                        Ok(Token::new(TokenType::Assign, self.line, col))
                    }
                }
                '<' => {
                    if let Some(&'=') = self.peek() {
                        self.advance();
                        Ok(Token::new(TokenType::LessEq, self.line, col))
                    } else {
                        Ok(Token::new(TokenType::Less, self.line, col))
                    }
                }
                '>' => {
                    if let Some(&'=') = self.peek() {
                        self.advance();
                        Ok(Token::new(TokenType::GreaterEq, self.line, col))
                    } else {
                        Ok(Token::new(TokenType::Greater, self.line, col))
                    }
                }
                '+' => Ok(Token::new(TokenType::Plus, self.line, col)),
                '-' => Ok(Token::new(TokenType::Minus, self.line, col)),
                '*' => Ok(Token::new(TokenType::Star, self.line, col)),
                '/' => Ok(Token::new(TokenType::Slash, self.line, col)),
                '(' => Ok(Token::new(TokenType::LParen, self.line, col)),
                ')' => Ok(Token::new(TokenType::RParen, self.line, col)),
                '[' => Ok(Token::new(TokenType::LBracket, self.line, col)),
                ']' => Ok(Token::new(TokenType::RBracket, self.line, col)),
                '{' => Ok(Token::new(TokenType::LBrace, self.line, col)),
                '}' => Ok(Token::new(TokenType::RBrace, self.line, col)),
                ',' => Ok(Token::new(TokenType::Comma, self.line, col)),
                ':' => Ok(Token::new(TokenType::Colon, self.line, col)),
                '.' => Ok(Token::new(TokenType::Dot, self.line, col)),
                '"' => {
                    // Backtrack the advance for read_string
                    self.column -= 1; 
                    self.read_string()
                },
                _ if c.is_ascii_digit() => Ok(self.read_number(c)),
                _ if c.is_alphabetic() || c == '_' => Ok(self.read_identifier_or_keyword(c)),
                _ => Err(format!("Unexpected character '{}' at line {}, col {}", c, self.line, col)),
            }
        } else {
            Ok(Token::new(TokenType::Eof, self.line, self.column))
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = token.token_type == TokenType::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }
}
