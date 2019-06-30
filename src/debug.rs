use std::convert::TryInto;

use crate::chunk::{read_u24, Chunk, OpCode};
use crate::value::Value;

pub fn disassemble(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);

    let mut offset = 0;
    while offset < chunk.len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);

    if offset > 0 && chunk.get_line_number(offset) == chunk.get_line_number(offset - 1) {
        print!("   | ");
    } else {
        print!("{:4} ", chunk.get_line_number(offset));
    }

    let instruction: Result<OpCode, String> = chunk.code[offset].try_into();
    match instruction {
        Ok(op @ OpCode::OpConstant) => constant_instruction(chunk, op.to_string().as_str(), offset),
        Ok(op @ OpCode::OpConstantLong) => {
            constant_long_instruction(chunk, op.to_string().as_str(), offset)
        }
        Ok(op @ _) => simple_instruction(op.to_string().as_str(), offset),
        Err(err) => {
            println!("Unknown opcode {}", err);
            offset + 1
        }
    }
}

fn constant_instruction(chunk: &Chunk, name: &str, offset: usize) -> usize {
    assert!(chunk.code.len() > offset + 1);
    let constant = chunk.code[offset + 1];
    print!("{:>-16} {:4} '", name, constant);
    assert!(chunk.constants.len() > constant as usize);
    print_value(&chunk.constants[constant as usize]);
    println!("'");
    offset + 2
}

fn constant_long_instruction(chunk: &Chunk, name: &str, offset: usize) -> usize {
    assert!(chunk.code.len() > offset + 1);
    let constant = read_u24(&chunk.code[offset + 1..=offset + 3]);
    print!("{:>-16} {:4} '", name, constant);
    assert!(chunk.constants.len() > constant as usize);
    print_value(&chunk.constants[constant as usize]);
    println!("'");
    offset + 4
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{:>-16}", name);
    return offset + 1;
}

pub(crate) fn print_value(value: &Value) {
    print!("{}", value);
}
