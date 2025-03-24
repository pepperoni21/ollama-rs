use proc_macro::TokenStream;

mod function;

#[proc_macro_attribute]
pub fn function(attr: TokenStream, value: TokenStream) -> TokenStream {
    function::function_impl(attr, value)
}
