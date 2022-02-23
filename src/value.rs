use crate::{
    errors::{JabroniError, JabroniResult},
    utils,
};
type Number = i32;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(Number),
    Boolean(bool),
    String(String),
    Void,
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

    pub fn compare(&mut self, value: Value) -> JabroniResult {
        self.assert_same_type(&value)?;
        if self == &value {
            *self = Value::Boolean(true);
        } else {
            *self = Value::Boolean(false);
        }
        Ok(())
    }
}
