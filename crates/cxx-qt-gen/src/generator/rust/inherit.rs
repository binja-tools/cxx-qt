// SPDX-FileCopyrightText: 2023 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Leon Matthes <leon.matthes@kdab.com>

// SPDX-License-Identifier: MIT OR Apache-2.0

use quote::{quote, quote_spanned};
use syn::{Item, Result, spanned::Spanned};

use crate::{
    generator::rust::fragment::GeneratedRustFragment,
    parser::inherit::ParsedInheritedMethod,
};
use crate::syntax::attribute::attribute_take_path;

pub fn generate(
    methods: &[ParsedInheritedMethod],
) -> Result<GeneratedRustFragment> {
    let mut blocks = GeneratedRustFragment::default();

    let mut bridges = methods
        .iter()
        .map(|method| {
            let wrapper_ident_str = method.wrapper_ident().to_string();

            // Remove any cxx_name attribute on the original method
            // As we need it to use the wrapper ident
            let original_method = {
                let mut original_method = method.method.clone();
                attribute_take_path(&mut original_method.attrs, &["cxx_name"]);
                original_method
            };

            let mut unsafe_block = None;
            let mut unsafe_call = Some(quote! { unsafe });
            if method.safe {
                std::mem::swap(&mut unsafe_call, &mut unsafe_block);
            }
            syn::parse2(quote_spanned! {
                method.method.span() =>
                #unsafe_block extern "C++" {
                    #[cxx_name = #wrapper_ident_str]
                    #original_method
                }
            })
        })
        .collect::<Result<Vec<Item>>>()?;

    blocks.cxx_mod_contents.append(&mut bridges);
    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use syn::{ForeignItemFn, parse_quote};

    use crate::{
        syntax::safety::Safety,
        tests::assert_tokens_eq,
    };

    use super::*;

    fn generate_from_foreign(
        method: ForeignItemFn,
        safety: Safety,
    ) -> Result<GeneratedRustFragment> {
        let inherited_methods = vec![ParsedInheritedMethod::parse(method, safety).unwrap()];
        generate(&inherited_methods)
    }

    #[test]
    fn test_mutable() {
        let generated = generate_from_foreign(
            parse_quote! {
                fn test(self: Pin<&mut MyObject>, a: B, b: C);
            },
            Safety::Safe,
        )
        .unwrap();

        assert_eq!(generated.cxx_mod_contents.len(), 1);
        assert_eq!(generated.cxx_qt_mod_contents.len(), 0);

        assert_tokens_eq(
            &generated.cxx_mod_contents[0],
            quote! {
                unsafe extern "C++" {
                    #[cxx_name = "testCxxQtInherit"]
                    fn test(self: Pin<&mut MyObject>, a: B, b: C);
                }
            },
        );
    }

    #[test]
    fn test_immutable() {
        let generated = generate_from_foreign(
            parse_quote! {
                fn test(self: &MyObject, a: B, b: C);
            },
            Safety::Safe,
        )
        .unwrap();

        assert_eq!(generated.cxx_mod_contents.len(), 1);
        assert_eq!(generated.cxx_qt_mod_contents.len(), 0);

        assert_tokens_eq(
            &generated.cxx_mod_contents[0],
            quote! {
                unsafe extern "C++" {
                    #[cxx_name = "testCxxQtInherit"]
                    fn test(self: &MyObject, a: B, b: C);
                }
            },
        );
    }

    #[test]
    fn test_unsafe() {
        let generated = generate_from_foreign(
            parse_quote! {
                unsafe fn test(self: &MyObject);
            },
            Safety::Unsafe,
        )
        .unwrap();

        assert_eq!(generated.cxx_mod_contents.len(), 1);
        assert_eq!(generated.cxx_qt_mod_contents.len(), 0);

        assert_tokens_eq(
            &generated.cxx_mod_contents[0],
            quote! {
                extern "C++" {
                    #[cxx_name = "testCxxQtInherit"]
                    unsafe fn test(self: &MyObject);
                }
            },
        );
    }
}
