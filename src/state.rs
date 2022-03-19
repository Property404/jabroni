use crate::{
    binding::{Binding, BindingMap},
    errors::{JabroniError, JabroniResult},
    value::Subroutine,
    Value,
};
use pest::{iterators::Pair, Parser};
use std::rc::Rc;

#[derive(Parser)]
#[grammar = "jabroni.pest"]
struct IdentParser;

#[derive(Default)]
pub struct Jabroni {
    bindings: BindingMap,
}

impl Jabroni {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define_constant(&mut self, ident: &str, value: Value) -> JabroniResult {
        self.define_binding(ident, value, false)
    }

    pub fn define_variable(&mut self, ident: &str, value: Value) -> JabroniResult {
        self.define_binding(ident, value, true)
    }

    pub fn update_variable(&mut self, ident: &str, value: Value) -> JabroniResult {
        self.bindings.get_mut(ident)?.set_value(value)?;

        Ok(())
    }

    fn define_binding(&mut self, ident: &str, value: Value, mutable: bool) -> JabroniResult {
        let ident = ident.to_string();
        if self.bindings.has(&ident) {
            return Err(JabroniError::DoubleDefinition(format!(
                "Cannot define '{ident}' because it has already been defined"
            )));
        }
        self.bindings.set(ident, Binding::new(value, mutable));
        Ok(())
    }

    pub fn run_expression(&mut self, code: &str) -> JabroniResult<Value> {
        let mut pairs = IdentParser::parse(Rule::jabroni_expression, code)
            .map_err(|e| JabroniError::Parse(format!("{}", e)))?;

        self.interpret_expression(pairs.next().unwrap())
    }

    pub fn run_script(&mut self, code: &str) -> JabroniResult<Value> {
        let pairs = IdentParser::parse(Rule::jabroni_script, code)
            .map_err(|e| JabroniError::Parse(format!("{}", e)))?;

        let mut value = Value::Null;
        for pair in pairs {
            match pair.as_rule() {
                Rule::statement => {
                    value = self.interpret_statement(pair)?;
                }
                Rule::EOI => (),
                _ => panic!("Unexpected rule found while running script"),
            }
        }
        Ok(value)
    }

    fn interpret_lvalue<'a>(
        pair: Pair<Rule>,
        bindings: &'a mut BindingMap,
    ) -> JabroniResult<&'a mut Binding> {
        match pair.as_rule() {
            Rule::ident => bindings.get_mut(pair.as_str()),
            Rule::kernel => Self::interpret_lvalue(pair.into_inner().next().unwrap(), bindings),
            Rule::member_access => {
                let mut pair = pair.into_inner();
                let object: &mut BindingMap = bindings
                    .get_mut(pair.next().unwrap().as_str())?
                    .value_mut()
                    .as_object_mut()
                    .ok_or_else(|| JabroniError::Type("Not an object".into()))?;
                Self::interpret_lvalue(pair.next().unwrap(), object)
            }
            _ => Err(JabroniError::Parse(format!(
                "Cannot make out lvalue expression: {}",
                pair.as_str()
            ))),
        }
    }

    fn interpret_expression(&mut self, pair: Pair<Rule>) -> JabroniResult<Value> {
        match pair.as_rule() {
            Rule::ident => Ok(self.bindings.get(pair.as_str())?.value().clone()),
            Rule::member_access => {
                let lvalue = Self::interpret_lvalue(pair, &mut self.bindings)?;
                Ok(lvalue.value().clone())
            }

            Rule::function_call => {
                let mut pair = pair.into_inner();
                let subroutine = Self::interpret_lvalue(pair.next().unwrap(), &mut self.bindings)?
                    .value()
                    .as_subroutine()
                    .ok_or_else(|| JabroniError::Type("Not a function".into()))?
                    .clone();

                let mut args = Vec::new();
                for arg in pair {
                    args.push(self.interpret_expression(arg)?);
                }

                let callback = &subroutine.callback;
                callback(&mut args)
            }
            Rule::ternary => {
                let mut pair = pair.into_inner();
                let condition = self.interpret_expression(pair.next().unwrap())?;
                match condition {
                    Value::Boolean(condition) => {
                        if !condition {
                            pair.next().unwrap();
                        }
                        self.interpret_expression(pair.next().unwrap())
                    }
                    _ => Err(JabroniError::Type(
                        "Ternary condition must be boolean".into(),
                    )),
                }
            }
            Rule::string_literal => return Value::from_string_literal(pair.as_str()),
            Rule::numeric_literal => return Value::from_numeric_literal(pair.as_str()),
            Rule::boolean_literal => {
                return Value::from_boolean_literal(pair.as_str());
            }
            Rule::null_literal => Ok(Value::Null),
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
                    Self::interpret_lvalue(lhs, &mut self.bindings)?.set_value(operand)?;
                } else {
                    unimplemented!("Unimplemented assignment operator: {}", operator);
                }
                // Assignment return void because we don't want to accidentally assign while trying
                // to compare
                Ok(Value::Null)
            }
            Rule::comparison | Rule::sum | Rule::product => {
                let mut pairs = pair.into_inner();
                let mut value = self.interpret_expression(pairs.next().unwrap())?;
                while let Some(operator) = pairs.next() {
                    let operator = operator.as_str();
                    let operand = self.interpret_expression(pairs.next().unwrap())?;
                    if operator == "==" {
                        value.compare(operand, false)?;
                    } else if operator == "===" {
                        value.compare(operand, true)?;
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
                unimplemented!("Unimplemented expression rule: {:?}", pair.as_rule());
            }
        }
    }

    fn interpret_statement(&mut self, pair: Pair<Rule>) -> JabroniResult<Value> {
        match pair.as_rule() {
            Rule::expression => {
                self.interpret_expression(pair)?;
            }
            Rule::statement => return self.interpret_statement(pair.into_inner().next().unwrap()),
            Rule::block_statement => {
                let mut value = Value::Null;
                for pair in pair.into_inner() {
                    value = self.interpret_statement(pair)?;
                }
                return Ok(value);
            }
            Rule::function_statement => {
                let mut pair = pair.into_inner();

                let function_name = pair.next().unwrap();
                let mut params = Vec::new();
                for param in pair.next().unwrap().into_inner() {
                    params.push(param.as_str().to_string());
                }
                let num_args = params.len();

                let body = pair.next().unwrap().as_str().to_string();
                let callback = move |args: &mut [Value]| -> JabroniResult<Value> {
                    if args.len() != num_args {
                        return Err(JabroniError::InvalidArguments(
                            "Incorrect number of arguments".into(),
                        ));
                    }

                    // Copy params/args (WARN: currently pass by value only)
                    let mut substate = Jabroni::new();
                    for (param, arg) in params.iter().zip(args.iter_mut()) {
                        println!("Passing {param}");
                        substate
                            .bindings
                            .set(param.into(), Binding::constant(arg.clone()));
                    }

                    substate.run_script(body.as_str())
                };
                let subroutine = Subroutine {
                    number_of_args: num_args as u8,
                    callback: Rc::new(Box::new(callback)),
                };
                self.bindings.set(
                    function_name.as_str().into(),
                    Binding::constant(Value::Subroutine(subroutine)),
                );
            }
            Rule::return_statement => {
                return self.interpret_expression(pair.into_inner().next().unwrap());
            }
            _ => {
                unimplemented!("Unimplemented statement rule: {:?}", pair.as_rule());
            }
        }
        Ok(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_expressions() {
        let mut state = Jabroni::new();
        assert_eq!(state.run_expression("4").unwrap(), 4.into());
        assert_eq!(state.run_expression("2+2").unwrap(), 4.into());
        assert_eq!(state.run_expression("3*10-5").unwrap(), 25.into());
        assert_eq!(state.run_expression("1+(10-7)*3").unwrap(), 10.into());
        assert_eq!(state.run_expression("2+2==4").unwrap(), true.into());
        assert_eq!(state.run_expression("2+2==5").unwrap(), false.into());
        assert_eq!(state.run_expression("2+2==4==false").unwrap(), false.into());
        assert_eq!(state.run_expression("true==true").unwrap(), true.into());
        assert_eq!(state.run_expression("1==1?100:200").unwrap(), 100.into());
        assert_eq!(state.run_expression("1==2?100:200").unwrap(), 200.into());
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

    #[test]
    fn strings() {
        let mut state = Jabroni::new();
        state
            .define_variable("foo", Value::String("Hello World!".into()))
            .unwrap();

        state.run_expression("foo='Shut up!'").unwrap();
        assert_eq!(
            state.run_expression("foo").unwrap(),
            Value::String("Shut up!".into())
        );

        state.run_expression("foo='\\''").unwrap();
        assert_eq!(
            state.run_expression("foo").unwrap(),
            Value::String("\'".into())
        );

        state.run_expression("foo=\"Hello!\"").unwrap();
        assert_eq!(
            state.run_expression("foo").unwrap(),
            Value::String("Hello!".into())
        );

        state.run_expression("foo='\\n\\t\\r'").unwrap();
        assert_eq!(
            state.run_expression("foo").unwrap(),
            Value::String("\n\t\r".into())
        );
    }

    #[test]
    fn objects() {
        let mut state = Jabroni::new();

        let mut object = BindingMap::default();
        object.set("bar".into(), Binding::variable(Value::Number(8)));
        object.set("baz".into(), Binding::constant(Value::Number(42)));
        let object = Value::Object(object);
        state.define_variable("foo", object.clone()).unwrap();

        assert_eq!(state.run_expression("foo").unwrap(), object);
        assert_eq!(state.run_expression("foo.bar").unwrap(), Value::Number(8));
        assert_eq!(state.run_expression("foo.baz").unwrap(), Value::Number(42));

        state.run_expression("foo.bar=0").unwrap();
        assert!(state.run_expression("foo.baz=1").is_err());

        assert_eq!(state.run_expression("foo.bar").unwrap(), Value::Number(0));
        assert_eq!(state.run_expression("foo.baz").unwrap(), Value::Number(42));
    }

    #[test]
    fn object_method() {
        fn bar(_: &mut [Value]) -> JabroniResult<Value> {
            Ok(Value::Number(42))
        }

        let mut state = Jabroni::new();
        let mut object = BindingMap::default();
        object.set(
            "bar".into(),
            Binding::constant(Value::Subroutine(Subroutine {
                number_of_args: 0,
                callback: Rc::new(Box::new(bar)),
            })),
        );

        let object = Value::Object(object);
        state.define_variable("foo", object.clone()).unwrap();

        assert_eq!(state.run_expression("foo.bar()").unwrap(), 42.into());
    }

    #[test]
    fn call_rust_function() {
        let mut state = Jabroni::new();
        fn foo(_: &mut [Value]) -> JabroniResult<Value> {
            Ok(Value::Number(42))
        }
        state
            .define_constant(
                "foo",
                Value::Subroutine(Subroutine {
                    number_of_args: 0,
                    callback: Rc::new(Box::new(foo)),
                }),
            )
            .unwrap();
        assert_eq!(state.run_expression("foo()").unwrap(), 42.into());
    }

    #[test]
    fn call_jabroni_function() {
        let mut state = Jabroni::new();

        state.run_script("function foo() {return 42;}").unwrap();
        assert_eq!(state.run_expression("foo()").unwrap(), 42.into());

        state
            .run_script("function add_one(x) {return x+1;}")
            .unwrap();
        assert_eq!(state.run_expression("add_one(41)").unwrap(), 42.into());

        state
            .run_script("function add_together(x, y) {return x+y;}")
            .unwrap();
        assert_eq!(
            state.run_expression("add_together(9,10)").unwrap(),
            19.into()
        );
    }

    #[test]
    fn statements() {
        let mut state = Jabroni::new();
        state.define_variable("x", Value::Number(0)).unwrap();
        state.define_variable("y", Value::Number(0)).unwrap();
        state.run_script("x=0;y=1;x=2;\n").unwrap();
        assert_eq!(state.run_expression("x").unwrap(), 2.into());
        assert_eq!(state.run_expression("y").unwrap(), 1.into());
    }
}
