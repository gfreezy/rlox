use crate::chunk::{read_u24, Chunk, OpCode};
use crate::compiler::Compiler;
use crate::debug::{disassemble_instruction, print_value};
use crate::error::{self, Result};
use crate::value::Value;
use snafu::{OptionExt, ResultExt};
use std::convert::TryInto;

const STACK_MAX: usize = 100;

pub struct VM {
    pub chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::with_capacity(STACK_MAX),
        }
    }

    pub fn write(&mut self, byte: u8, line: u32) {
        self.chunk.write(byte, line)
    }

    pub fn write_opcode(&mut self, opcode: OpCode, line: u32) {
        self.chunk.write(opcode as u8, line)
    }

    pub fn write_constant(&mut self, value: Value, line: u32) {
        self.chunk.write_constant(value, line)
    }

    pub fn interpret_source(&mut self, source: &str) -> Result<bool> {
        let mut compilier = Compiler::new(source.as_bytes(), &mut self.chunk);
        let ret = compilier.compile()?;
        self.run()?;
        Ok(ret)
    }

    fn read_byte(&mut self) -> u8 {
        assert!(self.chunk.code.len() > self.ip);
        let byte = self.chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    fn read_constant(&mut self) -> Value {
        let constant = self.read_byte();
        assert!(self.chunk.constants.len() > constant as usize);
        self.chunk.constants[constant as usize].clone()
    }

    fn read_constant_long(&mut self) -> Value {
        let constant = read_u24(&[0, self.read_byte(), self.read_byte(), self.read_byte()]);
        assert!(self.chunk.constants.len() > constant as usize);
        self.chunk.constants[constant as usize].clone()
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    pub fn pop(&mut self) -> Result<Value> {
        self.stack
            .pop()
            .context(error::NoOpCodeError { msg: "pop error" })
    }

    pub fn peek(&self, index: usize) -> Result<&Value> {
        self.stack
            .get(self.stack.len() - index - 1)
            .context(error::NoOpCodeError {
                msg: format!("peek {}", index),
            })
    }

    fn run(&mut self) -> Result<()> {
        macro_rules! binary_op {
            ($op:expr, $ty:tt, $err_msg:expr) => {
                let left = self.pop()?.$ty().context(error::TypeError {
                    msg: $err_msg,
                    line: self.chunk.lines.get(self.ip) as usize,
                })?;
                let right = self.pop()?.$ty().context(error::TypeError {
                    msg: $err_msg,
                    line: self.chunk.lines.get(self.ip) as usize,
                })?;;
                self.push($op(right, left).into());
            };
            ($op:expr) => {
                let left = self.pop()?;
                let right = self.pop()?;
                self.push($op(right, left).into());
            };
        }

        loop {
            if cfg!(feature = "debug-trace-execution") {
                print!("      ");
                for slot in &self.stack {
                    print!("[ ");
                    print_value(slot);
                    print!(" ]");
                }
                println!();
                disassemble_instruction(&self.chunk, self.ip);
            }

            let instruction: OpCode = self.read_byte().try_into().expect("read byte");
            match instruction {
                OpCode::OpReturn => {
                    print_value(&self.pop()?);
                    println!();
                    return Ok(());
                }
                OpCode::OpConstant => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                OpCode::OpConstantLong => {
                    let constant = self.read_constant_long();
                    self.push(constant);
                }
                OpCode::OpNegate => {
                    let constant = self.pop()?;
                    self.push(
                        (-constant.to_number().context(error::TypeError {
                            msg: "no number value",
                            line: self.chunk.lines.get(self.ip) as usize,
                        })?)
                        .into(),
                    );
                }
                OpCode::OpAdd => {
                    if self.peek(0)?.is_str() && self.peek(1)?.is_str() {
                        binary_op!(|l, r| format!("{}{}", l, r), into_str, "not a str");
                    } else {
                        binary_op!(|l, r| l + r, into_number, "not a str");
                    }
                }
                OpCode::OpSubtract => {
                    binary_op!(|l, r| l - r, into_number, "not a number");
                }
                OpCode::OpMultiply => {
                    binary_op!(|l, r| l * r, into_number, "not a number");
                }
                OpCode::OpDivide => {
                    binary_op!(|l, r| l / r, into_number, "not a number");
                }
                OpCode::OpNil => {
                    self.push(Value::Nil);
                }
                OpCode::OpFalse => {
                    self.push(false.into());
                }
                OpCode::OpTrue => self.push(true.into()),
                OpCode::OpNot => {
                    let v = self.pop()?.is_falsey().into();
                    self.push(v)
                }
                OpCode::OpEqual => {
                    binary_op!(|l, r| l == r);
                }
                OpCode::OpGreater => {
                    binary_op!(|l, r| l > r, into_number, "not a number");
                }
                OpCode::OpLess => {
                    binary_op!(|l, r| l < r, into_number, "not a number");
                }
            }
        }
    }
}
