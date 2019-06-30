use std::fmt;
use std::ops::Deref;
use std::result::Result;

#[derive(Debug)]
pub struct ValueTypeError {
    msg: String,
}

impl fmt::Display for ValueTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "ValueTypeError: {}", self.msg)
    }
}

impl std::error::Error for ValueTypeError {}

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
}

impl Value {
    pub(crate) fn bool_value(&self) -> Result<bool, ValueTypeError> {
        match self {
            Value::Bool(v) => Ok(*v),
            _ => Err(ValueTypeError {
                msg: "Operand must be a bool".to_string(),
            }),
        }
    }

    pub(crate) fn is_bool(&self) -> bool {
        match self {
            Value::Bool(_) => true,
            _ => false,
        }
    }

    pub(crate) fn number_value(&self) -> Result<f64, ValueTypeError> {
        match self {
            Value::Number(v) => Ok(*v),
            _ => Err(ValueTypeError {
                msg: "Operand must be a number".to_string(),
            }),
        }
    }

    pub(crate) fn is_number(&self) -> bool {
        match self {
            Value::Number(_) => true,
            _ => false,
        }
    }

    pub(crate) fn is_nil(&self) -> bool {
        match self {
            Value::Nil => true,
            _ => false,
        }
    }

    pub(crate) fn is_falsey(&self) -> bool {
        self.is_nil() || (self.is_bool() && !self.bool_value().expect("not bool"))
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Number(v)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Value::Number(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Nil => write!(f, "nil"),
        }
    }
}
pub struct ConstArray(Vec<Value>);

impl ConstArray {
    pub fn new() -> Self {
        ConstArray(Vec::with_capacity(100))
    }

    pub fn write(&mut self, value: Value) {
        self.0.push(value)
    }
}

impl Deref for ConstArray {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
