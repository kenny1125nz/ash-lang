use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<Value>),
    Nil,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::String(_) => "string",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::Array(_) => "array",
            Value::Nil => "nil",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(false) | Value::Nil => false,
            Value::Int(0) => false,
            Value::Float(f) if *f == 0.0 => false,
            Value::String(s) if s.is_empty() || s == "0" || s == "false" => false,
            Value::Array(a) if a.is_empty() => false,
            _ => true,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    fn is_numeric(&self) -> bool {
        matches!(self, Value::Int(_) | Value::Float(_))
    }

    pub fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Array(a), Value::Array(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                Ok(Value::Array(result))
            }
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            _ if matches!(self, Value::String(_)) || matches!(other, Value::String(_)) => {
                Ok(Value::String(format!("{}{}", self, other)))
            }
            _ => Err(format!("+ not supported between {} and {}", self.type_name(), other.type_name())),
        }
    }

    pub fn sub(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
            _ => Err(format!("- not supported between {} and {}", self.type_name(), other.type_name())),
        }
    }

    pub fn mul(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
            _ => Err(format!("* not supported between {} and {}", self.type_name(), other.type_name())),
        }
    }

    pub fn div(&self, other: &Value) -> Result<Value, String> {
        match (self.as_f64(), other.as_f64()) {
            (Some(a), Some(b)) => {
                if b == 0.0 {
                    Err("division by zero".to_string())
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            _ => Err(format!("/ not supported between {} and {}", self.type_name(), other.type_name())),
        }
    }

    pub fn rem(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 % b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a % *b as f64)),
            _ => Err(format!("% not supported between {} and {}", self.type_name(), other.type_name())),
        }
    }

    pub fn neg(&self) -> Result<Value, String> {
        match self {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(format!("- requires numeric operand, got {}", self.type_name())),
        }
    }

    pub fn eq(&self, other: &Value) -> Result<Value, String> {
        Ok(Value::Bool(self.values_equal(other)))
    }

    pub fn ne(&self, other: &Value) -> Result<Value, String> {
        Ok(Value::Bool(!self.values_equal(other)))
    }

    fn values_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.values_equal(y))
            }
            _ if self.is_numeric() && other.is_numeric() => {
                self.as_f64().unwrap() == other.as_f64().unwrap()
            }
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => format!("{}", self) == format!("{}", other),
        }
    }

    pub fn lt(&self, other: &Value) -> Result<Value, String> {
        if let (Some(a), Some(b)) = (self.as_f64(), other.as_f64()) {
            return Ok(Value::Bool(a < b));
        }
        if let (Value::String(a), Value::String(b)) = (self, other) {
            return Ok(Value::Bool(a < b));
        }
        Err(format!("< not supported between {} and {}", self.type_name(), other.type_name()))
    }

    pub fn gt(&self, other: &Value) -> Result<Value, String> {
        if let (Some(a), Some(b)) = (self.as_f64(), other.as_f64()) {
            return Ok(Value::Bool(a > b));
        }
        if let (Value::String(a), Value::String(b)) = (self, other) {
            return Ok(Value::Bool(a > b));
        }
        Err(format!("> not supported between {} and {}", self.type_name(), other.type_name()))
    }

    pub fn le(&self, other: &Value) -> Result<Value, String> {
        if let (Some(a), Some(b)) = (self.as_f64(), other.as_f64()) {
            return Ok(Value::Bool(a <= b));
        }
        if let (Value::String(a), Value::String(b)) = (self, other) {
            return Ok(Value::Bool(a <= b));
        }
        Err(format!("<= not supported between {} and {}", self.type_name(), other.type_name()))
    }

    pub fn ge(&self, other: &Value) -> Result<Value, String> {
        if let (Some(a), Some(b)) = (self.as_f64(), other.as_f64()) {
            return Ok(Value::Bool(a >= b));
        }
        if let (Value::String(a), Value::String(b)) = (self, other) {
            return Ok(Value::Bool(a >= b));
        }
        Err(format!(">= not supported between {} and {}", self.type_name(), other.type_name()))
    }

    pub fn len(&self) -> Result<Value, String> {
        match self {
            Value::String(s) => Ok(Value::Int(s.len() as i64)),
            Value::Array(a) => Ok(Value::Int(a.len() as i64)),
            _ => Err(format!("len() not supported for {}", self.type_name())),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Int(i)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}
