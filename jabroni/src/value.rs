use crate::{
    binding::BindingMap,
    errors::{JabroniError, JabroniResult},
    utils,
};
use enum_as_inner::EnumAsInner;
use std::{
    fmt::{Debug, Display, Formatter},
    rc::Rc,
};

type Number = i32;

type SubroutineCallback = Box<dyn Fn(&mut [Value]) -> JabroniResult<Value>>;
#[derive(Clone)]
pub struct Subroutine {
    pub number_of_args: u8,
    pub callback: Rc<SubroutineCallback>,
}

impl Subroutine {
    pub fn new(number_of_args: u8, callback: SubroutineCallback) -> Self {
        Self {
            number_of_args,
            callback: Rc::new(callback),
        }
    }
}

impl Debug for Subroutine {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "[function]")
    }
}
impl PartialEq for Subroutine {
    fn eq(&self, other: &Self) -> bool {
        self.number_of_args == other.number_of_args && Rc::ptr_eq(&self.callback, &other.callback)
    }
}

#[derive(PartialEq, Debug, Clone, EnumAsInner)]
pub enum Value {
    Number(Number),
    Boolean(bool),
    String(String),
    Object(BindingMap),
    Subroutine(Subroutine),
    Null,
}

impl Value {
    pub fn from_string_literal(literal: &str) -> JabroniResult<Self> {
        Ok(Value::String(utils::unquote(literal)?))
    }

    pub fn from_numeric_literal(literal: &str) -> JabroniResult<Self> {
        Ok(Value::Number(
            literal
                .to_string()
                .parse::<i32>()
                .map_err(|e| JabroniError::Parse(e.to_string()))?,
        ))
    }

    pub fn from_boolean_literal(literal: &str) -> JabroniResult<Self> {
        if literal == "true" {
            Ok(Value::Boolean(true))
        } else if literal == "false" {
            Ok(Value::Boolean(false))
        } else {
            Err(JabroniError::Parse(format!(
                "Couldn't form boolean literal from '{}'",
                literal
            )))
        }
    }

    fn unwrap_into_number(self) -> JabroniResult<Number> {
        match self {
            Value::Number(value) => Ok(value),
            _ => Err(JabroniError::Type("Expected number".into())),
        }
    }

    fn unwrap_as_number(&mut self) -> JabroniResult<&mut Number> {
        match self {
            Value::Number(value) => Ok(value),
            _ => Err(JabroniError::Type("Expected number".into())),
        }
    }

    pub fn assert_same_type(&self, value: &Value) -> JabroniResult {
        if std::mem::discriminant(self) != std::mem::discriminant(value) {
            return Err(JabroniError::Type("Type mismatch".into()));
        }
        Ok(())
    }

    pub fn add(&mut self, value: Value) -> JabroniResult {
        *self.unwrap_as_number()? += value.unwrap_into_number()?;
        Ok(())
    }

    pub fn subtract(&mut self, value: Value) -> JabroniResult {
        *self.unwrap_as_number()? -= value.unwrap_into_number()?;
        Ok(())
    }

    pub fn multiply(&mut self, value: Value) -> JabroniResult {
        *self.unwrap_as_number()? *= value.unwrap_into_number()?;
        Ok(())
    }

    pub fn inverse(&mut self) -> JabroniResult {
        match self {
            Self::Boolean(boolean) => *self = Self::Boolean(!*boolean),
            _ => {
                return Err(JabroniError::Type("Cannot inverse a non-boolean".into()));
            }
        }
        Ok(())
    }

    pub fn compare(&mut self, value: Value, allow_type_diff: bool) -> JabroniResult {
        if std::mem::discriminant(self) != std::mem::discriminant(&value) {
            *self = false.into();
            if allow_type_diff {
                return Ok(());
            } else {
                return Err(JabroniError::Type(
                    "Cannot compare between values of different types. Try using '===' or '!=='"
                        .into(),
                ));
            }
        }

        if matches!(value, Value::Null) && !allow_type_diff {
            return Err(JabroniError::Type(
                "Can't compare null values. Use '===' or '!=='".into(),
            ));
        }

        let comparison = match self {
            Value::Boolean(v) => v == value.as_boolean().unwrap(),
            Value::Number(v) => v == value.as_number().unwrap(),
            Value::String(v) => v == value.as_string().unwrap(),
            Value::Null => true,
            _ => {
                return Err(JabroniError::Type(
                    "Cannot compare values of this type".into(),
                ));
            }
        };
        *self = Value::Boolean(comparison);
        Ok(())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Value {
        Value::Boolean(value)
    }
}

impl From<Number> for Value {
    fn from(value: Number) -> Value {
        Value::Number(value)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Value::Number(value) => write!(f, "{}", value),
            Value::Boolean(value) => write!(f, "{}", value),
            Value::String(value) => write!(f, "{}", value),
            Value::Null => write!(f, "null"),
            // These aren't consistent with JavaScript
            Value::Object(_) => write!(f, "[function]"),
            Value::Subroutine(_) => write!(f, "[object]"),
        }
    }
}
