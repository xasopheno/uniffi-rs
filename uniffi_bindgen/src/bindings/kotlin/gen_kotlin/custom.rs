/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use askama::Template;

use super::{filters, CustomTypeConfig};
use crate::backend::{CodeDeclaration, CodeOracle, CodeType, Literal};
use crate::interface::{FFIType, Type};

pub struct CustomCodeType {
    name: String,
    builtin: Type,
    config: Option<CustomTypeConfig>,
}

impl CustomCodeType {
    pub fn new(name: String, builtin: Type, config: Option<CustomTypeConfig>) -> Self {
        CustomCodeType {
            name,
            builtin,
            config,
        }
    }

    fn ffi_converter_name(&self, oracle: &dyn CodeOracle) -> String {
        format!("FfiConverter{}", self.canonical_name(oracle))
    }
}

impl CodeType for CustomCodeType {
    fn type_label(&self, oracle: &dyn CodeOracle) -> String {
        match &self.config {
            // We have a custom type config use the supplied type name from the config
            Some(custom_type_config) => custom_type_config
                .type_name
                .clone()
                .unwrap_or_else(|| self.name.clone()),
            // No custom type config, use our builtin type
            None => self.builtin.type_label(oracle),
        }
    }

    fn canonical_name(&self, _oracle: &dyn CodeOracle) -> String {
        format!("Type{}", self.name.clone())
    }

    fn literal(&self, _oracle: &dyn CodeOracle, _literal: &Literal) -> String {
        unreachable!("Can't have a literal of a custom type");
    }

    fn lower(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!(
            "{}.lower({})",
            self.ffi_converter_name(oracle),
            oracle.var_name(nm)
        )
    }

    fn write(
        &self,
        oracle: &dyn CodeOracle,
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
    ) -> String {
        format!(
            "{}.write({}, {})",
            self.ffi_converter_name(oracle),
            oracle.var_name(nm),
            target
        )
    }

    fn lift(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.lift({})", self.ffi_converter_name(oracle), nm)
    }

    fn read(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.read({})", self.ffi_converter_name(oracle), nm)
    }

    fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(format!(
            "// Helper code for {} is found in CustomTypeTemplate.kt",
            self.name,
        ))
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "CustomTypeTemplate.kt")]
pub struct KotlinCustomType {
    name: String,
    builtin: Type,
    config: Option<CustomTypeConfig>,
}

impl KotlinCustomType {
    pub fn new(name: String, builtin: Type, config: Option<CustomTypeConfig>) -> Self {
        Self {
            name,
            builtin,
            config,
        }
    }

    fn type_name(&self, config: &CustomTypeConfig) -> String {
        config
            .type_name
            .clone()
            .unwrap_or_else(|| self.name.clone())
    }

    fn builtin_ffi_type(&self) -> FFIType {
        (&self.builtin).into()
    }
}

impl CodeDeclaration for KotlinCustomType {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }

    fn imports(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        match &self.config {
            None => None,
            Some(custom_type_config) => custom_type_config.imports.clone(),
        }
    }
}
