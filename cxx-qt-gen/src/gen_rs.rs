use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::extract::QObject;

/// Generate Rust code that used CXX to interact with the C++ code generated for a QObject
pub fn generate_qobject_cxx(obj: &QObject) -> Result<TokenStream, TokenStream> {
    let class_name = &obj.ident;
    let rust_class_name = &obj.rust_struct_ident;

    // TODO: Abstract this calculation to make it common to gen_rs and gen_cpp
    let ident_snake = class_name.to_string().to_case(Case::Snake);
    let import_path = format!("cxx-qt-gen/include/{}.h", ident_snake);

    let cpp_functions: Vec<TokenStream> = Vec::new();
    let mut rs_functions = Vec::new();

    // Invokables are only added to extern rust side
    for i in &obj.invokables {
        let ident = &i.ident;
        let parameters = &i.parameters;

        if parameters.is_empty() {
            rs_functions.push(quote! {
                fn #ident(self: &#rust_class_name);
            });
        } else {
            let mut parameters_quotes = Vec::new();
            for p in parameters {
                let ident = &p.ident;
                let type_ident = &p.type_ident.ident;
                if p.type_ident.is_ref {
                    parameters_quotes.push(quote! {
                        #ident: &#type_ident
                    });
                } else {
                    parameters_quotes.push(quote! {
                        #ident: #type_ident
                    });
                }
            }

            // TODO: add cpp functions for the invokable so that it can be called
            // consider how the different types for strings work here

            rs_functions.push(quote! {
                fn #ident(self: &#rust_class_name, #(#parameters_quotes, )*);
            });
        }
    }

    // TODO: add properties getter/setter these will be in both rust and C++ sides?

    let new_object_ident = format_ident!("new_{}", class_name);
    let create_object_ident = format_ident!("create_{}_rs", ident_snake);

    let output = quote! {
        #[cxx::bridge]
        mod ffi {
            unsafe extern "C++" {
                include!(#import_path);

                type #class_name;

                #(#cpp_functions)*

                fn #new_object_ident() -> UniquePtr<#class_name>;
            }

            extern "Rust" {
                type #rust_class_name;

                #(#rs_functions)*

                fn #create_object_ident() -> Box<#rust_class_name>;
            }
        }
    };
    Ok(output.into_token_stream())
}

/// Generate a Rust function that heap constructs the Rust object corresponding to the QObject
fn generate_rust_object_creator(obj: &QObject) -> Result<TokenStream, TokenStream> {
    let class_name = &obj.ident;
    let rust_class_name = &obj.rust_struct_ident;

    let ident_snake = class_name.to_string().to_case(Case::Snake);
    let fn_ident = format_ident!("create_{}_rs", ident_snake);

    // TODO: check if the original object had an explicit constructor and if so ensure that the create
    // function also takes the same parameters so that it can call this constructor. The C++ object will
    // also need to take the same parameters in its constructor. If the object is not default constructable
    // and does not provide a constructor then we need to throw an error.

    let output = quote! {
        fn #fn_ident() -> Box<#rust_class_name> {
            Box::new(#rust_class_name {})
        }
    };
    Ok(output.into_token_stream())
}

/// Generate all the Rust code required to communicate with a QObject backed by generated C++ code
pub fn generate_qobject_rs(obj: &QObject) -> Result<TokenStream, TokenStream> {
    let mod_ident = &obj.module_ident;
    let cxx_block = generate_qobject_cxx(obj)?;

    let rust_class_name = &obj.rust_struct_ident;
    let mut renamed_struct = obj.original_struct.clone();
    renamed_struct.ident = rust_class_name.clone();

    // TODO: also pull in the functions for properties
    let methods = obj.invokables.iter().map(|m| &m.original_method);

    let creator_fn = generate_rust_object_creator(obj)?;

    let output = quote! {
        mod #mod_ident {
            #cxx_block

            #renamed_struct

            impl #rust_class_name {
                #(#methods)*
            }

            #creator_fn
        }
    };
    Ok(output.into_token_stream())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract_qobject;

    use pretty_assertions::assert_eq;
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    use syn::ItemMod;

    fn format_rs_source(rs_code: &str) -> String {
        // NOTE: this error handling is pretty rough so should only used for tests
        let mut command = Command::new("rustfmt");
        let mut child = command
            .args(&["--emit", "stdout"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        // Scope stdin to force an automatic flush
        {
            let mut stdin = child.stdin.take().unwrap();
            write!(stdin, "{}", rs_code).unwrap();
        }

        let output = child.wait_with_output().unwrap();
        let output = String::from_utf8(output.stdout).unwrap();

        // Quote does not retain empty lines so we throw them away in the case of the
        // reference string as to not cause clashes
        output.replace("\n\n", "\n")
    }

    #[test]
    fn generates_basic_only_invokables() {
        // TODO: we probably want to parse all the test case files we have
        // only once as to not slow down different tests on the same input.
        // This can maybe be done with some kind of static object somewhere.
        let source = include_str!("../test_inputs/basic_only_invokable.rs");
        let module: ItemMod = syn::parse_str(source).unwrap();
        let qobject = extract_qobject(module).unwrap();

        let expected_output = include_str!("../test_outputs/basic_only_invokable.rs");
        let expected_output = format_rs_source(expected_output);

        let generated_rs = generate_qobject_rs(&qobject).unwrap().to_string();
        let generated_rs = format_rs_source(&generated_rs);

        assert_eq!(generated_rs, expected_output);
    }

    // TODO: add tests for more complex cases such as invokables with parameters
    // and for objects with properties
}