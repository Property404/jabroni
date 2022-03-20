use crate::{
    errors::{JabroniError, JabroniResult},
    value::Value,
};
use std::{
    collections::HashMap,
    fmt::{Debug, Error, Formatter},
};

#[derive(Debug, Clone, PartialEq)]
/// Value type used in a [BindingMap] to represent variables or constants.
pub struct Binding {
    mutable: bool,
    value: Value,
}

impl Binding {
    /// Construct a new binding.
    pub const fn new(value: Value, mutable: bool) -> Self {
        Self { mutable, value }
    }

    /// Construct a new constant binding.
    pub const fn constant(value: Value) -> Self {
        Self {
            mutable: false,
            value,
        }
    }

    /// Construct a new variable binding.
    pub const fn variable(value: Value) -> Self {
        Self {
            mutable: true,
            value,
        }
    }

    /// Return binding mutability.
    pub const fn mutable(&self) -> bool {
        self.mutable
    }

    /// Return binding value
    pub const fn value(&self) -> &Value {
        &self.value
    }

    /// Return mutable reference to binding value
    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    /// Set binding value
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
/// A mapping(really a collection of mappings) between identifiers and [Binding]s.
/// Has multiple layers, one for each scope.
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
    /// Copy this [BindingMap] and add a new scope
    pub fn clone_with_new_scope(&self) -> Self {
        let mut clone = self.clone();
        clone.maps.push(Default::default());
        clone
    }

    /// Check if the identifier exists in the current scope.
    pub fn has_on_top(&self, ident: &str) -> bool {
        debug_assert!(!self.maps.is_empty());
        if self.maps[self.maps.len() - 1].get(ident).is_some() {
            return true;
        }
        false
    }

    /// Map an identifier to a binding.
    pub fn set(&mut self, ident: String, value: Binding) {
        debug_assert!(!self.maps.is_empty());
        let length = self.maps.len();
        self.maps[length - 1].insert(ident, value);
    }

    /// Get binding from map.
    pub fn get(&self, ident: &str) -> JabroniResult<&Binding> {
        debug_assert!(!self.maps.is_empty());
        for map in self.maps.iter().rev() {
            if let Some(entry) = map.get(ident) {
                return Ok(entry);
            }
        }
        Err(JabroniError::Reference(format!("'{ident}' does not exist")))
    }

    /// Get mutable reference to binding.
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
