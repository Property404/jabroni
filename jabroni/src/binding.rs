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

#[derive(Clone)]
pub struct BindingMap {
    maps: Vec<HashMap<String, Binding>>,
}

impl Default for BindingMap {
    fn default() -> Self {
        Self {
            maps: vec![HashMap::default()],
        }
    }
}

impl BindingMap {
    pub fn new_context(&self) -> Self {
        let mut clone = self.clone();
        clone.maps.push(Default::default());
        clone
    }

    pub fn has_on_top(&self, ident: &str) -> bool {
        debug_assert!(!self.maps.is_empty());
        if self.maps[self.maps.len() - 1].get(ident).is_some() {
            return true;
        }
        false
    }

    pub fn set(&mut self, ident: String, value: Binding) {
        debug_assert!(!self.maps.is_empty());
        let length = self.maps.len();
        self.maps[length - 1].insert(ident, value);
    }

    pub fn get(&self, ident: &str) -> JabroniResult<&Binding> {
        debug_assert!(!self.maps.is_empty());
        for map in self.maps.iter().rev() {
            if let Some(entry) = map.get(ident) {
                return Ok(entry);
            }
        }
        Err(JabroniError::Reference(format!("'{ident}' does not exist")))
    }

    pub fn get_mut(&mut self, ident: &str) -> JabroniResult<&mut Binding> {
        debug_assert!(!self.maps.is_empty());
        for map in self.maps.iter_mut().rev() {
            if let Some(entry) = map.get_mut(ident) {
                return Ok(entry);
            }
        }
        Err(JabroniError::Reference(format!("'{ident}' does not exist")))
    }
}

impl Debug for BindingMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{{")?;
        for map in self.maps.iter() {
            write!(f, "\t{{")?;
            for (ident, binding) in map {
                write!(f, "\t\t\"{ident}\": {binding:?})")?;
            }
            write!(f, "\t}},")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl PartialEq for BindingMap {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
