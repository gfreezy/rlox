use paste;
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

macro_rules! impl_enum_variant {
    ($name:tt, $enum_ty:tt, $variant:tt, $ty:ty) => {
        impl $enum_ty {
            paste::item! {
                #[allow(dead_code)]
                pub(crate) fn [<to_ $name>](&self) -> Result<$ty, ValueTypeError> {
                    match self {
                        $enum_ty::$variant(v) => Ok(v.clone()),
                        _ => Err(ValueTypeError {
                            msg: format!("Operand must be a {}", stringify!($ty)),
                        }),
                    }
                }

                #[allow(dead_code)]
                pub(crate) fn [<into_ $name>](self) -> Result<$ty, ValueTypeError> {
                    match self {
                        $enum_ty::$variant(v) => Ok(v),
                        _ => Err(ValueTypeError {
                            msg: format!("Operand must be a {}", stringify!($ty)),
                        }),
                    }
                }

                #[allow(dead_code)]
                pub(crate) fn [<as_ $name>](&self) -> Result<&$ty, ValueTypeError> {
                    match self {
                        $enum_ty::$variant(v) => Ok(v),
                        _ => Err(ValueTypeError {
                            msg: format!("Operand must be a {}", stringify!($ty)),
                        }),
                    }
                }

                #[allow(dead_code)]
                pub(crate) fn [<is_ $name>](&self) -> bool {
                    match self {
                        $enum_ty::$variant(_) => true,
                        _ => false,
                    }
                }
            }
        }

        impl From<$ty> for $enum_ty {
            fn from(v: $ty) -> Self {
                $enum_ty::$variant(v)
            }
        }
    };
}

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    Str(String),
}

impl_enum_variant!(bool, Value, Bool, bool);
impl_enum_variant!(number, Value, Number, f64);
impl_enum_variant!(str, Value, Str, String);

impl Value {
    pub(crate) fn is_nil(&self) -> bool {
        match self {
            Value::Nil => true,
            _ => false,
        }
    }

    pub(crate) fn is_falsey(&self) -> bool {
        self.is_nil() || (self.is_bool() && !self.to_bool().expect("not bool"))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Value::Number(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Nil => write!(f, "nil"),
            Value::Str(s) => write!(f, "{}", s),
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
