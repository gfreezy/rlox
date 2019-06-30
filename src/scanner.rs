use crate::error::{self, Result};
use crate::token_type::TokenType;
use snafu::OptionExt;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub(crate) ty: TokenType,
    pub(crate) lexeme: Vec<u8>,
    pub(crate) line: usize,
}

pub struct Scanner<'a> {
    source: &'a [u8],
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        self.start = self.current;

        let c = if let Some(c) = self.advance() {
            c
        } else {
            return Ok(self.make_token(TokenType::Eof));
        };

        if c.is_ascii_digit() {
            return self.number().context(error::ScanError {
                msg: "invalid number",
                line: self.line,
            });
        }

        if c.is_ascii_alphabetic() {
            return self.identifier().context(error::ScanError {
                msg: "invalid identifier",
                line: self.line,
            });
        }

        let token = match c {
            b'(' => self.make_token(TokenType::LeftParen),
            b')' => self.make_token(TokenType::RightParen),
            b'{' => self.make_token(TokenType::LeftBrace),
            b'}' => self.make_token(TokenType::RightBrace),
            b';' => self.make_token(TokenType::Semicolon),
            b':' => self.make_token(TokenType::Colon),
            b',' => self.make_token(TokenType::Comma),
            b'.' => self.make_token(TokenType::Dot),
            b'-' => self.make_token(TokenType::Minus),
            b'+' => self.make_token(TokenType::Plus),
            b'/' => self.make_token(TokenType::Slash),
            b'?' => self.make_token(TokenType::QuestionMark),
            b'*' => self.make_token(TokenType::Star),
            b'!' => {
                if self.match_and_advance(b'=') {
                    self.make_token(TokenType::BangEqual)
                } else {
                    self.make_token(TokenType::Bang)
                }
            }
            b'=' => {
                if self.match_and_advance(b'=') {
                    self.make_token(TokenType::EqualEqual)
                } else {
                    self.make_token(TokenType::Equal)
                }
            }
            b'<' => {
                if self.match_and_advance(b'=') {
                    self.make_token(TokenType::LessEqual)
                } else {
                    self.make_token(TokenType::Less)
                }
            }
            b'>' => {
                if self.match_and_advance(b'=') {
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    self.make_token(TokenType::Greater)
                }
            }
            b'"' => self.string()?,

            _ => {
                return error::ScanError {
                    msg: "unknown token",
                    line: self.current,
                }
                .fail()
            }
        };
        return Ok(token);
    }

    fn is_at_end(&self) -> bool {
        self.current == self.source.len()
    }

    fn make_token(&self, ty: TokenType) -> Token {
        Token {
            ty,
            lexeme: self.source[self.start..self.current].to_vec(),
            line: self.line,
        }
    }

    fn advance(&mut self) -> Option<u8> {
        let c = self.source.get(self.current).copied();
        if c.is_some() {
            self.current += 1;
        }
        c
    }

    fn match_and_advance(&mut self, expected: u8) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.peek() != Some(expected) {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> Option<u8> {
        self.source.get(self.current).copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.source.get(self.current + 1).copied()
    }

    fn skip_whitespace(&mut self) -> Option<()> {
        loop {
            let c = self.peek()?;
            match c {
                b' ' | b'\r' | b'\t' => {
                    self.advance()?;
                }
                b'\n' => {
                    self.line += 1;
                    self.advance()?;
                }
                b'/' => {
                    if self.peek_next()? == b'/' {
                        while self.peek()? != b'\n' {
                            self.advance()?;
                        }
                    } else {
                        return None;
                    }
                }
                _ => return None,
            };
        }
    }

    fn string(&mut self) -> Result<Token> {
        while let Some(c) = self.peek() {
            if c != b'"' {
                if c == b'\n' {
                    self.line += 1;
                }
                self.advance();
            }
        }

        if self.advance() == Some(b'"') {
            Ok(self.make_token(TokenType::Str))
        } else {
            error::ScanError {
                msg: "Unterminated string",
                line: self.line,
            }
            .fail()
        }
    }

    fn number(&mut self) -> Option<Token> {
        while self.peek()?.is_ascii_digit() {
            self.advance()?;
        }

        if self.peek()? == b'.' && self.peek_next()?.is_ascii_digit() {
            self.advance()?;

            while self.peek()?.is_ascii_digit() {
                self.advance()?;
            }
        }
        Some(self.make_token(TokenType::Number))
    }

    fn identifier(&mut self) -> Option<Token> {
        loop {
            let c = self.peek()?;
            if c.is_ascii_alphabetic() || c.is_ascii_digit() {
                self.advance()?;
            } else {
                break;
            }
        }

        Some(self.make_token(self.identifier_type()))
    }

    fn identifier_type(&self) -> TokenType {
        match &self.source[self.start..self.current] {
            b"and" => TokenType::And,
            b"class" => TokenType::Class,
            b"else" => TokenType::Else,
            b"if" => TokenType::If,
            b"nil" => TokenType::Nil,
            b"or" => TokenType::Or,
            b"print" => TokenType::Print,
            b"return" => TokenType::Return,
            b"super" => TokenType::Super,
            b"var" => TokenType::Var,
            b"while" => TokenType::While,
            b"false" => TokenType::False,
            b"for" => TokenType::For,
            b"fun" => TokenType::Fun,
            b"this" => TokenType::This,
            b"true" => TokenType::True,
            _ => TokenType::Identifier,
        }
    }
}
