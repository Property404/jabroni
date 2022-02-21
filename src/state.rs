use crate::{
    errors::{JabroniError, JabroniResult},
    Value,
};
use std::collections::HashMap;

use pest::{iterators::Pair, Parser};

#[derive(Parser)]
#[grammar = "jabroni.pest"]
struct IdentParser;

#[derive(Clone)]
struct Binding {
    mutable: bool,
    value: Value,
}

impl Binding {
    const fn mutable(&self) -> bool {
        self.mutable
    }

    fn into_value(self) -> Value {
        self.value
    }
}

pub struct Jabroni {
    bindings: HashMap<String, Binding>,
}

impl Default for Jabroni {
    fn default() -> Self {
        Self::new()
    }
}

impl Jabroni {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn define_constant(&mut self, ident: &str, value: Value) -> JabroniResult {
        self.define_binding(ident, value, false)
    }

    pub fn define_variable(&mut self, ident: &str, value: Value) -> JabroniResult {
        self.define_binding(ident, value, true)
    }

    pub fn update_variable(&mut self, ident: &str, value: Value) -> JabroniResult {
        let ident = ident.to_string();
        let binding = self
            .bindings
            .get_mut(&ident)
            .ok_or_else(|| JabroniError::Reference(format!("'{ident}' does not exist")))?;

        if !binding.mutable() {
            return Err(JabroniError::Type(format!(
                "Cannot assign to '{ident}' because it is constant"
            )));
        }

        if std::mem::discriminant(&binding.value) != std::mem::discriminant(&value) {
            return Err(JabroniError::Type(ident));
        }

        binding.value = value;

        Ok(())
    }

    fn define_binding(&mut self, ident: &str, value: Value, mutable: bool) -> JabroniResult {
        let ident = ident.to_string();
        if self.bindings.get(&ident).is_some() {
            return Err(JabroniError::DoubleDefinition(format!(
                "Cannot define '{ident}' because it has already been defined"
            )));
        }
        self.bindings.insert(ident, Binding { mutable, value });
        Ok(())
    }

    pub fn run_expression(&mut self, code: &str) -> JabroniResult<Value> {
        let mut pairs = IdentParser::parse(Rule::jabroni, code)
            .map_err(|e| JabroniError::Parse(format!("{}", e)))?;

        self.interpret_expression(pairs.next().unwrap())
    }

    fn get_binding(&self, name: &str) -> JabroniResult<Binding> {
        let name = String::from(name);
        self.bindings
            .get(&name)
            .ok_or(JabroniError::Reference(name))
            .map(Clone::clone)
    }

    fn interpret_expression(&mut self, pair: Pair<Rule>) -> JabroniResult<Value> {
        match pair.as_rule() {
            Rule::ident => Ok(self.get_binding(pair.as_str())?.into_value()),
            Rule::numeric_literal => return Value::from_numeric_literal(pair.as_str()),
            Rule::boolean_literal => {
                return Value::from_boolean_literal(pair.as_str());
            }
            Rule::expression => {
                return self.interpret_expression(pair.into_inner().next().unwrap());
            }
            Rule::assignment => {
                let mut pairs = pair.into_inner();
                let lhs = pairs.next().unwrap();

                let operator = pairs.next().unwrap();
                let operator = operator.as_str();
                let operand = self.interpret_expression(pairs.next().unwrap())?;
                if operator == "=" {
                    self.update_variable(lhs.as_str(), operand)?;
                } else {
                    unimplemented!("Unimplemented assignment operator: {}", operator);
                }
                // Assignment return void because we don't want to accidentally assign while trying
                // to compare
                Ok(Value::Void)
            }
            Rule::comparison | Rule::sum | Rule::product => {
                let mut pairs = pair.into_inner();
                let mut value = self.interpret_expression(pairs.next().unwrap())?;
                while let Some(operator) = pairs.next() {
                    let operator = operator.as_str();
                    let operand = self.interpret_expression(pairs.next().unwrap())?;
                    if operator == "==" {
                        value.compare(operand)?;
                    } else if operator == "+" {
                        value.add(operand)?;
                    } else if operator == "-" {
                        value.subtract(operand)?;
                    } else if operator == "*" {
                        value.multiply(operand)?;
                    } else {
                        unimplemented!("Unimplemented operator: {}", operator);
                    }
                }
                Ok(value)
            }
            _ => {
                unimplemented!("Unimplemented rule: {:?}", pair.as_rule());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_expressions() {
        let mut state = Jabroni::new();
        assert_eq!(state.run_expression("4").unwrap(), Value::Number(4));
        assert_eq!(state.run_expression("2+2").unwrap(), Value::Number(4));
        assert_eq!(state.run_expression("3*10-5").unwrap(), Value::Number(25));
        assert_eq!(
            state.run_expression("1+(10-7)*3").unwrap(),
            Value::Number(10)
        );
        assert_eq!(
            state.run_expression("2+2==4").unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            state.run_expression("2+2==5").unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            state.run_expression("2+2==4==false").unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(
            state.run_expression("true==true").unwrap(),
            Value::Boolean(true)
        );
    }

    #[test]
    fn forbid_type_mismatch() {
        let mut state = Jabroni::new();
        state.define_variable("x", Value::Number(4)).unwrap();
        assert!(state.run_expression("4==false").is_err());
        assert!(state.run_expression("true==4").is_err());
        assert!(state.run_expression("x=true").is_err());
        // assignments return void
        assert!(state.run_expression("4==(x=4)").is_err());
    }

    #[test]
    fn constants() {
        let mut state = Jabroni::new();
        state.define_constant("FOO", Value::Number(42)).unwrap();
        assert_eq!(state.run_expression("FOO").unwrap(), Value::Number(42));
        assert_eq!(state.run_expression("FOO*10").unwrap(), Value::Number(420));
        // Constants can't be modified
        assert!(state.run_expression("FOO=8").is_err());
        // Cannot be defined multiple times
        assert!(state.define_constant("FOO", Value::Number(42)).is_err());
        assert!(state.define_variable("FOO", Value::Number(42)).is_err());
        // Cannot be updated
        assert!(state.update_variable("FOO", Value::Number(42)).is_err());
    }

    #[test]
    fn variables() {
        let mut state = Jabroni::new();
        state.define_variable("foo", Value::Number(42)).unwrap();

        assert_eq!(state.run_expression("foo").unwrap(), Value::Number(42));

        // Assign
        state.run_expression("foo=420").unwrap();
        assert_eq!(state.run_expression("foo").unwrap(), Value::Number(420));

        // Try to define twice
        assert!(state.define_variable("foo", Value::Number(42)).is_err());
        assert!(state.define_constant("foo", Value::Number(42)).is_err());

        // Update
        state.update_variable("foo", Value::Number(16)).unwrap();
        assert_eq!(state.run_expression("foo").unwrap(), Value::Number(16));
    }
}
