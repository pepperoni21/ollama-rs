use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, Expr, Token};

pub fn tool_group_impl(input: TokenStream) -> TokenStream {
    let expr_array = parse_macro_input!(input with Punctuated::<Expr, Token![,]>::parse_terminated);

    if expr_array.is_empty() {
        return TokenStream::from(quote! {
            ()
        });
    }

    let nested_tuples = expr_array
        .into_iter()
        .rev()
        .reduce(|acc, expr| syn::parse_quote!((#expr, #acc)))
        .unwrap();

    let expanded = quote! {
        #nested_tuples
    };

    TokenStream::from(expanded)
}
