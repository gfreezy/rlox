use std::ops::Deref;

pub type Value = f64;

pub struct ValueArray(Vec<Value>);

impl ValueArray {
    pub fn new() -> Self {
        ValueArray(Vec::with_capacity(100))
    }

    pub fn write(&mut self, value: Value) {
        self.0.push(value)
    }
}

impl Deref for ValueArray {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
