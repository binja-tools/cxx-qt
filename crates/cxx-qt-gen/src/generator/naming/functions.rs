// SPDX-FileCopyrightText: 2023 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Leon Matthes <leon.matthes@kdab.com>

// SPDX-License-Identifier: MIT OR Apache-2.0

use convert_case::{Case, Casing};
use quote::format_ident;
use syn::{Attribute, Ident};

use crate::syntax::attribute::attribute_find_path;
use crate::syntax::expr::expr_to_string;

use super::CombinedIdent;

impl CombinedIdent {
    /// Generate a CombinedIdent from a rust function name.
    /// C++ will use the CamelCase version of the function name.
    pub fn from_rust_function(attrs: &[Attribute], ident: &Ident) -> Self {
        // set default cpp name, based of convention in #828
        // https://github.com/KDAB/cxx-qt/issues/828#issuecomment-1920288043
        // println!("Custom backtrace: {}", Backtrace::force_capture());

        // rust:
        // - rust_name attribute
        // - if cxx_name exists: as_is
        // - snake_case
        // c++:
        // - cxx_name attribute
        // - if rust_name exists: as_is
        // - camelCase
        // in theory differentiate between extern C++Qt and extern RustQt
        // TODO: different defaults for C++Qt and RustQt

        let mut res = Self {
            cpp: format_ident!("{}", ident.to_string().to_case(Case::Camel)),
            rust: format_ident!("{}", ident.to_string().to_case(Case::Snake)),
        };

        let mut cxx_name_found = false;

        // Find any cxx_name
        if let Some(index) = attribute_find_path(attrs, &["cxx_name"]) {
            if let Ok(name_value) = &attrs[index].meta.require_name_value() {
                if let Ok(value_str) = expr_to_string(&name_value.value) {
                    cxx_name_found = true;
                    res.cpp = format_ident!("{value_str}");
                    res.rust = ident.clone();
                }
            }
        }

        // Find any rust_name
        if let Some(index) = attribute_find_path(attrs, &["rust_name"]) {
            if let Ok(name_value) = &attrs[index].meta.require_name_value() {
                if let Ok(value_str) = expr_to_string(&name_value.value) {
                    res.rust = format_ident!("{value_str}");
                    if !cxx_name_found {
                        res.cpp = ident.clone();
                    }
                }
            }
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use syn::{ForeignItemFn, parse_quote};

    use super::*;

    #[test]
    fn test_from_rust_function() {
        let method: ForeignItemFn = parse_quote! {
            extern "C++Qt"
            fn Test_Function();
        };
        let combined = CombinedIdent::from_rust_function(&method.attrs, &method.sig.ident);
        assert_eq!(combined.cpp, format_ident!("testFunction"));
        assert_eq!(combined.rust, format_ident!("test_function"));
    }

    #[test]
    fn test_from_rust_function_cxx_name() {
        let method: ForeignItemFn = parse_quote! {
            #[cxx_name = "TestFunction"]
            fn Test_Function();
        };
        let combined = CombinedIdent::from_rust_function(&method.attrs, &method.sig.ident);
        assert_eq!(combined.cpp, format_ident!("TestFunction"));
        assert_eq!(combined.rust, format_ident!("Test_Function"));
    }

    #[test]
    fn test_from_rust_function_rust_name() {
        let method: ForeignItemFn = parse_quote! {
            #[rust_name = "Test_Function"]
            fn TestFunction();
        };
        let combined = CombinedIdent::from_rust_function(&method.attrs, &method.sig.ident);
        assert_eq!(combined.cpp, format_ident!("TestFunction"));
        assert_eq!(combined.rust, format_ident!("Test_Function"));
    }

    #[test]
    fn test_from_rust_function_both() {
        let method: ForeignItemFn = parse_quote! {
            #[cxx_name = "TestFunction"]
            #[rust_name = "Test_Function"]
            fn test_function();
        };
        let combined = CombinedIdent::from_rust_function(&method.attrs, &method.sig.ident);
        assert_eq!(combined.cpp, format_ident!("TestFunction"));
        assert_eq!(combined.rust, format_ident!("Test_Function"));
    }
}
