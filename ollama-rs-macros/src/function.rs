//! This code is heavily inspired by [`gpt_function` implementation](https://docs.rs/gpt_fn_macros/1.0.0/src/gpt_fn_macros/proc/attr.rs.html).

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote_spanned, ToTokens};
use syn::{
    spanned::Spanned as _, Error, Expr, ExprLit, FnArg, Ident, ItemFn, Lit, Meta, MetaNameValue,
    Pat, Type,
};

pub fn function_impl(_attr: TokenStream, value: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(value as ItemFn);

    if input.sig.asyncness.is_none() {
        return Error::new_spanned(input.sig.fn_token, "function must be async")
            .to_compile_error()
            .into();
    }

    if !input.sig.generics.params.is_empty() {
        return Error::new_spanned(
            input.sig.generics,
            "functions with generics are not supported, as you'd need to provide a custom description for each combination of generics",
        )
        .to_compile_error()
        .into();
    }

    let Some(docs) = extract_docs(&input) else {
        return Error::new_spanned(input.sig.fn_token, "function must be documented")
            .to_compile_error()
            .into();
    };

    let vis = &input.vis;
    let function_name = &input.sig.ident;
    let function_module_name = Ident::new(&format!("__{}_data", input.sig.ident), input.span());

    let params_struct = match build_params_struct(&input, &docs) {
        Ok(params_struct) => params_struct,
        Err(err) => return err.to_compile_error().into(),
    };

    let tool_impl = build_tool_impl(
        &input,
        &docs,
        &params_struct,
        function_name,
        &function_module_name,
    );

    let function_params_struct_definition = &params_struct.tokens;

    quote_spanned!(input.span() =>
        #[doc(hidden)]
        mod #function_module_name {
            #[allow(unused_imports)]
            use super::*;

            use ollama_rs::re_exports::schemars;
            use ollama_rs::re_exports::serde;

            #function_params_struct_definition
        }

        #[allow(non_camel_case_types)]
        #vis struct #function_name;

        #tool_impl
    )
    .into()
}

fn build_tool_impl(
    input: &ItemFn,
    docs: &FunctionDocs,
    params_struct: &ParamsStruct,
    function_name: &Ident,
    function_module_name: &Ident,
) -> TokenStream2 {
    let function_name_str = function_name.to_string();
    let function_body = &input.block;
    let function_description = &docs.description;

    let function_params_struct_name = &params_struct.name;
    let function_params_struct_field_names = params_struct.fields.iter().map(|field| &field.name);

    quote_spanned!(input.span() =>
        impl ::ollama_rs::generation::tools::Tool for #function_name {
            type Params = #function_module_name::#function_params_struct_name;

            #[inline]
            fn name() -> &'static str {
                #function_name_str
            }

            #[inline]
            fn description() -> &'static str {
                #function_description
            }

            async fn call(&mut self, Self::Params { #(#function_params_struct_field_names),* }: Self::Params) -> ::std::result::Result<
                ::std::string::String,
                ::std::boxed::Box<dyn ::std::error::Error + Send + Sync>,
            > {
                #function_body
            }
        }
    )
}

fn build_params_struct(input: &ItemFn, docs: &FunctionDocs) -> syn::Result<ParamsStruct> {
    let name = Ident::new(
        &format!("__{}__Params", input.sig.ident),
        input.sig.ident.span(),
    );
    let span = input.span();
    let fields = prepare_params_struct_fields(input, docs)?;
    let tokens = quote_spanned!(span =>
        #[doc(hidden)]
        #[allow(non_camel_case_types, missing_docs)]
        #[derive(serde::Deserialize, schemars::JsonSchema)]
        #[serde(crate="ollama_rs::re_exports::serde")]
        pub struct #name {
            #(#fields)*
        }
    );

    Ok(ParamsStruct {
        name,
        fields,
        tokens,
    })
}

struct ParamsStruct {
    name: Ident,
    fields: Vec<ParamsField>,
    tokens: TokenStream2,
}

struct ParamsField {
    name: Ident,
    ty: Type,
    docs: String,
}

impl ToTokens for ParamsField {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let ty = &self.ty;
        let docs = &self.docs;

        tokens.extend(quote_spanned!(self.name.span() =>
            #[doc = #docs]
            pub #name: #ty,
        ));
    }
}

fn prepare_params_struct_fields(
    input: &ItemFn,
    docs: &FunctionDocs,
) -> syn::Result<Vec<ParamsField>> {
    input
        .sig
        .inputs
        .iter()
        .map(|arg| {
            let pat_type = match arg {
                FnArg::Receiver(_) => {
                    return Err(Error::new_spanned(arg, "self argument is not allowed"))
                }
                FnArg::Typed(pat_type) => pat_type,
            };

            let Pat::Ident(name) = &*pat_type.pat else {
                return Err(Error::new_spanned(
                    pat_type,
                    "only named arguments are allowed, e.g. `a: i32`",
                ));
            };
            let name_str = name.ident.to_string();
            let docs = docs
                .parameter_docs
                .get(&name_str)
                .cloned()
                .unwrap_or(name_str);

            Ok(ParamsField {
                name: name.ident.clone(),
                ty: *pat_type.ty.clone(),
                docs,
            })
        })
        .collect()
}

fn extract_docs(input: &ItemFn) -> Option<FunctionDocs> {
    let docs = input
        .attrs
        .iter()
        .filter_map(|attr| {
            if !attr.path().is_ident("doc") {
                return None;
            }

            let meta = &attr.meta;

            if let Meta::NameValue(MetaNameValue {
                value:
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(str), ..
                    }),
                ..
            }) = meta
            {
                return Some(str.value());
            }

            None
        })
        .collect::<Vec<String>>();

    let mut parameter_docs = HashMap::with_capacity(4);

    let mut lines = docs
        .iter()
        .flat_map(|a| a.split("\n"))
        .map(str::trim)
        .skip_while(|s| s.is_empty())
        .filter(|line| {
            if line.starts_with('*') && line.contains('-') {
                let (param_name, param_doc) = line[1..].trim().split_once('-').unwrap();
                let param_name = param_name.trim();
                let param_doc = param_doc.trim();
                parameter_docs.insert(param_name.to_owned(), param_doc.to_owned());
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    if let Some(&"") = lines.last() {
        lines.pop();
    }

    let joined = lines.join("\n");

    if joined.is_empty() {
        None
    } else {
        Some(FunctionDocs {
            description: joined,
            parameter_docs,
        })
    }
}

#[derive(Debug, Clone)]
struct FunctionDocs {
    description: String,
    parameter_docs: HashMap<String, String>,
}
