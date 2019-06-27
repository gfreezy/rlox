use std::collections::HashMap;

use snafu::{OptionExt, ResultExt};

use lazy_static::lazy_static;

use crate::chunk::Chunk;
use crate::chunk::OpCode::{OpAdd, OpDivide, OpMultiply, OpNegate, OpReturn, OpSubtract};
use crate::debug::disassemble;
use crate::error::{self, Error, Result};
use crate::scanner::{Scanner, Token};
use crate::token_type::TokenType;
use crate::value::Value;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryInto;

type ParseFn = fn(&mut Compiler) -> Result<()>;

#[derive(Clone)]
struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

impl ParseRule {
    fn prefix(&self) -> Result<ParseFn> {
        self.prefix.clone().context(error::ParseError)
    }

    fn infix(&self) -> Result<ParseFn> {
        self.infix.clone().context(error::ParseError)
    }
}

macro_rules! parse_rule {
    ( { $( { $ty:expr, $prefix:expr, $infix:expr, $precedence:expr }),* $(,)? } ) => {{
        let mut map = HashMap::new();
        $(map.insert(
            $ty,
            ParseRule {
                prefix: $prefix,
                infix: $infix,
                precedence: $precedence,
            },
        );)*
        map
    }};
}

lazy_static! {
    static ref RULES: HashMap<TokenType, ParseRule> = parse_rule!(
        {
            { TokenType::LeftParen, Some(grouping), None,    Precedence::Call },
            { TokenType::RightParen, None,     None,    Precedence::None },
            { TokenType::LeftBrace, None,     None,    Precedence::None },
            { TokenType::RightBrace, None,     None,    Precedence::None },
            { TokenType::Comma, None,     None,    Precedence::None },
            { TokenType::Dot, None,     None,    Precedence::Call },
            { TokenType::Minus, Some(unary),    Some(binary),  Precedence::Term },
            { TokenType::Plus, None,     Some(binary),  Precedence::Term },
            { TokenType::Semicolon, None,     None,    Precedence::None },
            { TokenType::Slash, None,     Some(binary),  Precedence::Factor },
            { TokenType::Star, None,     Some(binary),  Precedence::Factor },
            { TokenType::Bang, None,     None,    Precedence::None },
            { TokenType::BangEqual, None,     None,    Precedence::Equality },
            { TokenType::Equal, None,     None,    Precedence::None },
            { TokenType::EqualEqual, None,     None,    Precedence::Equality },
            { TokenType::Greater, None,     None,    Precedence::Comparison },
            { TokenType::GreaterEqual, None,     None,    Precedence::Comparison },
            { TokenType::Less, None,     None,    Precedence::Comparison },
            { TokenType::LessEqual, None,     None,    Precedence::Comparison },
            { TokenType::Identifier, None,     None,    Precedence::None },
            { TokenType::Str, None,     None,    Precedence::None },
            { TokenType::Number, Some(number),   None,    Precedence::None },
            { TokenType::And, None,     None,    Precedence::And },
            { TokenType::Class, None,     None,    Precedence::None },
            { TokenType::Else, None,     None,    Precedence::None },
            { TokenType::False, None,     None,    Precedence::None },
            { TokenType::For, None,     None,    Precedence::None },
            { TokenType::Fun, None,     None,    Precedence::None },
            { TokenType::If, None,     None,    Precedence::None },
            { TokenType::Nil, None,     None,    Precedence::None },
            { TokenType::Or, None,     None,    Precedence::Or },
            { TokenType::Print, None,     None,    Precedence::None },
            { TokenType::Return, None,     None,    Precedence::None },
            { TokenType::Super, None,     None,    Precedence::None },
            { TokenType::This, None,     None,    Precedence::None },
            { TokenType::True, None,     None,    Precedence::None },
            { TokenType::Var, None,     None,    Precedence::None },
            { TokenType::While, None,     None,    Precedence::None },
            { TokenType::Eof, None,     None,    Precedence::None },
        }
    );
}

fn get_rule(ty: TokenType) -> ParseRule {
    RULES.get(&ty).expect("no rule").clone()
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
enum TokenPosition {
    Current,
    Previous,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum Precedence {
    None,
    Assignment,
    // =
    Or,
    // or
    And,
    // and
    Equality,
    // == !=
    Comparison,
    // < > <= >=
    Term,
    // + -
    Factor,
    // * /
    Unary,
    // ! -
    Call,
    // . () []
    Primary,
}

struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Option<Token>,
    previous: Option<Token>,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    fn new(scanner: Scanner<'a>) -> Self {
        Parser {
            scanner,
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false,
        }
    }

    fn error_at_current(&mut self, msg: &str) -> Result<()> {
        self.error_at(TokenPosition::Current, msg)
    }

    fn error(&mut self, msg: &str) -> Result<()> {
        self.error_at(TokenPosition::Previous, msg)
    }

    fn error_at(&mut self, pos: TokenPosition, msg: &str) -> Result<()> {
        if self.panic_mode {
            return Ok(());
        }
        self.panic_mode = true;

        let token = match pos {
            TokenPosition::Current => self.current(),
            TokenPosition::Previous => self.previous(),
        }?;

        eprint!("[line {}] Error", token.line);
        match token.ty {
            TokenType::Eof => {
                eprint!(" at end");
            }
            _ => eprint!(" at '{}'", String::from_utf8_lossy(&token.lexeme)),
        }
        eprintln!(": {}", msg);
        self.had_error = true;
        Ok(())
    }

    fn consume(&mut self, ty: TokenType, msg: &str) -> Result<()> {
        if self.current()?.ty == ty {
            self.advance()?;
            return Ok(());
        }

        self.error_at_current(msg)
    }

    fn advance(&mut self) -> Result<()> {
        std::mem::swap(&mut self.previous, &mut self.current);

        loop {
            match self.scanner.scan_token() {
                Ok(t) => {
                    self.current = Some(t);
                    return Ok(());
                }
                Err(Error::ErrorToken { msg, .. }) => {
                    self.error_at_current(&msg)?;
                }
                Err(e) => {
                    eprintln!("unknown error {:?}", e);
                }
            }
        }
    }

    fn previous(&self) -> Result<&Token> {
        self.previous.as_ref().context(error::ParseError)
    }

    fn current(&self) -> Result<&Token> {
        self.current.as_ref().context(error::ParseError)
    }

    fn line(&self) -> usize {
        self.previous().expect("no previous").line
    }
}

pub struct Compiler<'a, 'b> {
    parser: Parser<'a>,
    chunk: &'b mut Chunk,
}

impl<'a, 'b> Compiler<'a, 'b> {
    pub fn new(source: &'a [u8], chunk: &'b mut Chunk) -> Self {
        let scanner = Scanner::new(source);
        Compiler {
            parser: Parser::new(scanner),
            chunk,
        }
    }

    pub fn compile(&mut self) -> Result<bool> {
        self.parser.had_error = false;
        self.parser.panic_mode = false;

        self.parser.advance()?;
        expression(self)?;
        self.parser
            .consume(TokenType::Eof, "Expect end of expression")?;
        self.end()?;

        Ok(!self.parser.had_error)
    }

    fn emit_byte(&mut self, byte: u8) -> Result<()> {
        self.chunk.write(byte, self.parser.line() as u32);
        Ok(())
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) -> Result<()> {
        self.emit_byte(byte1)?;
        self.emit_byte(byte2)
    }

    fn emit_return(&mut self) -> Result<()> {
        self.emit_byte(OpReturn as u8)
    }

    fn end(&mut self) -> Result<()> {
        self.emit_return()?;

        if cfg!(feature = "debug-print-code") && !self.parser.had_error {
            disassemble(self.chunk, "code");
        }
        Ok(())
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<()> {
        self.parser.advance()?;
        let prefix_rule = get_rule(self.parser.previous()?.ty).prefix()?;
        prefix_rule(self)?;

        while precedence <= get_rule(self.parser.current()?.ty).precedence {
            self.parser.advance()?;
            let infix_rule = get_rule(self.parser.previous()?.ty).infix()?;
            infix_rule(self)?;
        }
        Ok(())
    }

    fn emit_constant(&mut self, value: Value) -> Result<()> {
        self.chunk
            .write_constant(value, self.parser.previous()?.line as u32);
        Ok(())
    }
}

fn expression(compiler: &mut Compiler) -> Result<()> {
    compiler.parse_precedence(Precedence::Assignment)?;
    Ok(())
}

fn number(compiler: &mut Compiler) -> Result<()> {
    let value: Value = String::from_utf8_lossy(&compiler.parser.previous()?.lexeme)
        .parse()
        .context(error::ParseFloatError)?;
    compiler.emit_constant(value)
}

fn grouping(compiler: &mut Compiler) -> Result<()> {
    expression(compiler)?;
    compiler
        .parser
        .consume(TokenType::RightParen, "Expect ')' after expression")
}

fn unary(compiler: &mut Compiler) -> Result<()> {
    let operator_type = compiler.parser.previous()?.ty;
    compiler.parse_precedence(Precedence::Unary)?;
    match operator_type {
        TokenType::Minus => compiler.emit_byte(OpNegate as u8),
        _ => unreachable!(),
    }
}

fn binary(compiler: &mut Compiler) -> Result<()> {
    let operator_type = compiler.parser.previous()?.ty;
    let rule = get_rule(operator_type);
    compiler.parse_precedence(
        (rule.precedence as u8 + 1)
            .try_into()
            .expect("invalid precedence"),
    )?;

    let code = match operator_type {
        TokenType::Plus => OpAdd,
        TokenType::Minus => OpSubtract,
        TokenType::Star => OpMultiply,
        TokenType::Slash => OpDivide,
        _ => unreachable!(),
    };
    compiler.emit_byte(code as u8)
}
