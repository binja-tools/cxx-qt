use quote::format_ident;
use syn::ForeignItemFn;

// SPDX-FileCopyrightText: 2022 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::{generator::naming::CombinedIdent, parser::method::ParsedMethod};

/// Names for parts of a method (which could be a Q_INVOKABLE)
pub struct QMethodName {
    pub name: CombinedIdent,
    pub wrapper: CombinedIdent,
}

impl From<&ParsedMethod> for QMethodName {
    fn from(invokable: &ParsedMethod) -> Self {
        Self::from(&invokable.method)
    }
}

impl From<&ForeignItemFn> for QMethodName {
    fn from(method: &ForeignItemFn) -> Self {
        let ident = &method.sig.ident;
        let name = CombinedIdent::from_rust_function(&method.attrs, &ident.clone());
        let wrapper = CombinedIdent::wrapper_from_invokable(&name);
        Self { name, wrapper}
    }
}

impl CombinedIdent {
    /// For a given ident generate the Rust and C++ wrapper names
    fn wrapper_from_invokable(ident: &CombinedIdent) -> Self {
        Self {
            cpp: format_ident!("{}Wrapper", ident.cpp),
            rust: format_ident!("{}_wrapper", ident.rust),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_from_impl_method() {
        let parsed = ParsedMethod {
            method: parse_quote! {
                fn my_invokable(self: &MyObject);
            },
            qobject_ident: format_ident!("MyObject"),
            mutable: false,
            safe: true,
            parameters: vec![],
            specifiers: HashSet::new(),
            is_qinvokable: true,
        };

        let invokable = QMethodName::from(&parsed);
        assert_eq!(invokable.name.cpp, format_ident!("myInvokable"));
        assert_eq!(invokable.name.rust, format_ident!("my_invokable"));
        assert_eq!(invokable.wrapper.cpp, format_ident!("myInvokableWrapper"));
        assert_eq!(
            invokable.wrapper.rust,
            format_ident!("my_invokable_wrapper")
        );
    }
}
