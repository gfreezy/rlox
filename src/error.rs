use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum Error {
    #[snafu(display("scan token error at line {}: {}", line, msg))]
    ErrorToken {
        line: usize,
        msg: String,
    },
    CompileError,
    RuntimeError,
    ParseError,
    ParseFloatError {
        source: std::num::ParseFloatError,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub(crate) fn error_token(msg: &str, line: usize) -> ErrorToken<usize, String> {
        ErrorToken {
            line,
            msg: msg.to_string(),
        }
    }
}
