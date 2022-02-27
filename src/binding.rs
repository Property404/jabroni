use crate::{
    errors::{JabroniError, JabroniResult},
    value::Value,
};
use std::{
    collections::HashMap,
    fmt::{Debug, Error, Formatter},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    mutable: bool,
    value: Value,
}

impl Binding {
    pub const fn new(value: Value, mutable: bool) -> Self {
        Self { mutable, value }
    }

    pub const fn constant(value: Value) -> Self {
        Self {
            mutable: false,
            value,
        }
    }

    pub const fn variable(value: Value) -> Self {
        Self {
            mutable: true,
            value,
        }
    }

    pub const fn mutable(&self) -> bool {
        self.mutable
    }

    pub const fn value(&self) -> &Value {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    pub fn set_value(&mut self, value: Value) -> JabroniResult {
        if std::mem::discriminant(self.value()) != std::mem::discriminant(&value) {
            return Err(JabroniError::Type(
                "Type mismatch in binding assignment".into(),
            ));
        }

        if !self.mutable() {
            return Err(JabroniError::Type(
                "Cannot mutably access binding because it is constant".into(),
            ));
        }
        self.value = value;
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct BindingMap(HashMap<String, Binding>);

impl BindingMap {
    pub fn has(&self, ident: &str) -> bool {
        self.0.get(ident).is_some()
    }

    pub fn set(&mut self, ident: String, value: Binding) {
        self.0.insert(ident, value);
    }

    pub fn get(&self, ident: &str) -> JabroniResult<&Binding> {
        self.0
            .get(ident)
            .ok_or_else(|| JabroniError::Reference(format!("'{ident}' does not exist")))
    }

    pub fn get_mut(&mut self, ident: &str) -> JabroniResult<&mut Binding> {
        let binding = self
            .0
            .get_mut(ident)
            .ok_or_else(|| JabroniError::Reference(format!("'{ident}' does not exist")))?;

        Ok(binding)
    }
}

impl Debug for BindingMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{{")?;
        for (ident, binding) in &self.0 {
            write!(f, "\"{ident}\": {binding:?})")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl PartialEq for BindingMap {
    fn eq(&self, other: &Self) -> bool {
        let map1 = &self.0;
        let map2 = &other.0;
        if map1.len() != map2.len() {
            return false;
        }

        for (key, value) in map1.iter() {
            if map2.get(key) != Some(value) {
                return false;
            }
        }

        true
    }
}
