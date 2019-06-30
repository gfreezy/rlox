use crate::value::ValueTypeError;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum Error {
    #[snafu(display("scan token error at line {}: {}", line, msg))]
    ScanError {
        line: usize,
        msg: String,
    },
    CompileError {
        line: usize,
        msg: String,
    },
    RuntimeError {
        msg: String,
    },
    TypeError {
        msg: String,
        line: usize,
        source: ValueTypeError,
    },
    ParseError {
        msg: String,
    },
    ParseRuleError {
        msg: String,
    },
    ParseFloatError {
        line: usize,
        msg: String,
        source: std::num::ParseFloatError,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
