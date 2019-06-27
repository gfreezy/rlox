use snafu::ResultExt;

use crate::chunk::{read_u24, Chunk, OpCode};
use crate::compiler::Compiler;
use crate::debug::{disassemble_instruction, print_value};
use crate::error::Result;
use crate::value::Value;
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
        self.chunk.constants[constant as usize]
    }

    fn read_constant_long(&mut self) -> Value {
        let constant = read_u24(&[0, self.read_byte(), self.read_byte(), self.read_byte()]);
        assert!(self.chunk.constants.len() > constant as usize);
        self.chunk.constants[constant as usize]
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    fn run(&mut self) -> Result<()> {
        macro_rules! binary_op {
            ($op:tt) => {
                let left = self.pop().expect("get left operand");
                let right = self.pop().expect("get left operand");
                self.push(right $op left);
            };
        }

        loop {
            if cfg!(feature = "debug-trace-execution") {
                print!("      ");
                for slot in &self.stack {
                    print!("[ ");
                    print_value(*slot);
                    print!(" ]");
                }
                println!();
                disassemble_instruction(&self.chunk, self.ip);
            }

            let offset = self.ip;
            let instruction: OpCode = self.read_byte().try_into().expect("read byte");
            match instruction {
                OpCode::OpReturn => {
                    print_value(self.pop().expect("empyt stack"));
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
                    let constant = self.pop().expect("get number");
                    self.push(-constant);
                }
                OpCode::OpAdd => {
                    binary_op!(+);
                }
                OpCode::OpSubtract => {
                    binary_op!(-);
                }
                OpCode::OpMultiply => {
                    binary_op!(*);
                }
                OpCode::OpDivide => {
                    binary_op!(/);
                }
            }
        }
        Ok(())
    }
}
