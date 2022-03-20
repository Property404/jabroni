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

type SubroutineCallback = Box<dyn Fn(BindingMap, &mut [Value]) -> JabroniResult<Value>>;

#[derive(Clone)]
/// A Jabroni function.
/// # Example
/// ```
/// use jabroni::{BindingMap, Jabroni, Subroutine, Value as JabroniValue};
///
/// let mut interpreter = Jabroni::new();
/// interpreter.define_constant(
///     "add_one",
///     JabroniValue::Subroutine(Subroutine::new(
///         1,
///         Box::new(|_context: BindingMap, args: &mut [JabroniValue]| {
///             Ok(JabroniValue::Number(*args[0].as_number().unwrap() + 1))
///         }),
///     )),
/// );
/// assert_eq!(interpreter.run_expression("add_one(1)").unwrap(), 2.into());
/// ```
pub struct Subroutine {
    number_of_args: Option<usize>,
    callback: Rc<SubroutineCallback>,
}

impl Subroutine {
    /// Construct a new Jabroni function.
    pub fn new(number_of_args: usize, callback: SubroutineCallback) -> Self {
        Self {
            number_of_args: Some(number_of_args),
            callback: Rc::new(callback),
        }
    }

    /// Construct a new Jabroni function that can take any number of arguments.
    pub fn new_variadic(callback: SubroutineCallback) -> Self {
        Self {
            number_of_args: None,
            callback: Rc::new(callback),
        }
    }

    /// Call the function.
    pub fn call(&self, context: BindingMap, args: &mut [Value]) -> JabroniResult<Value> {
        if let Some(number_of_args) = self.number_of_args {
            if args.len() != number_of_args {
                return Err(JabroniError::InvalidArguments(
                    "Incorrect number of arguments".into(),
                ));
            }
        }
        let callback = self.callback.clone();
        callback(context, args)
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
/// Enumeration of the different types in Jabroni.
pub enum Value {
    /// Number type
    Number(Number),
    /// Boolean type
    Boolean(bool),
    /// String type
    String(String),
    /// Object type
    Object(BindingMap),
    /// Function type
    Subroutine(Subroutine),
    /// Null type - corresponds to Javascript's Null/Undefined
    Null,
}

impl Value {
    /// Create a String value form a quoted string literal.
    ///
    /// #Example
    /// ```
    /// use jabroni::Value as JabroniValue;
    /// let value = JabroniValue::from_string_literal("\"hello\"").unwrap();
    /// assert_eq!(value, JabroniValue::String("hello".into()));
    /// ```
    pub fn from_string_literal(literal: &str) -> JabroniResult<Self> {
        Ok(Value::String(utils::unquote(literal)?))
    }

    /// Construct a new Number value from a numeric literal.
    ///
    /// #Example
    /// ```
    /// use jabroni::Value as JabroniValue;
    /// let value = JabroniValue::from_numeric_literal("42").unwrap();
    /// assert_eq!(value, JabroniValue::Number(42.into()));
    /// ```
    pub fn from_numeric_literal(literal: &str) -> JabroniResult<Self> {
        Ok(Value::Number(
            literal
                .to_string()
                .parse::<i32>()
                .map_err(|e| JabroniError::Parse(e.to_string()))?,
        ))
    }

    /// Construct a new Boolean value from a boolean literal.
    ///
    /// #Example
    /// ```
    /// use jabroni::Value as JabroniValue;
    /// let value = JabroniValue::from_boolean_literal("true").unwrap();
    /// assert_eq!(value, JabroniValue::Boolean(true.into()));
    /// ```
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

    /// Add a Number value
    pub fn add(&mut self, value: Value) -> JabroniResult {
        *self.unwrap_as_number()? += value.unwrap_into_number()?;
        Ok(())
    }

    /// Subtract a Number value
    pub fn subtract(&mut self, value: Value) -> JabroniResult {
        *self.unwrap_as_number()? -= value.unwrap_into_number()?;
        Ok(())
    }

    /// Multiply with a Number value
    pub fn multiply(&mut self, value: Value) -> JabroniResult {
        *self.unwrap_as_number()? *= value.unwrap_into_number()?;
        Ok(())
    }

    /// Negate the value (bools only)
    pub fn inverse(&mut self) -> JabroniResult {
        match self {
            Self::Boolean(boolean) => *self = Self::Boolean(!*boolean),
            _ => {
                return Err(JabroniError::Type("Cannot inverse a non-boolean".into()));
            }
        }
        Ok(())
    }

    /// Compare equality. `allow_type_diff` allows for comparisons between different types (always
    /// false)
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

    /// Compare with custom comparator. Type differences are not allowed
    pub fn compare_inequality(
        &mut self,
        value: Value,
        comparator: &dyn Fn(Number, Number) -> bool,
    ) -> JabroniResult {
        if std::mem::discriminant(self) != std::mem::discriminant(&value) {
            return Err(JabroniError::Type(
                "Cannot compare between values of different types. Try using '===' or '!=='".into(),
            ));
        }

        let comparison = match self {
            Value::Number(v) => comparator(*v, *value.as_number().unwrap()),
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
