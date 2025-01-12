use proc_macro::TokenStream;

mod function;
mod tool_group;

#[proc_macro_attribute]
pub fn function(attr: TokenStream, value: TokenStream) -> TokenStream {
    function::function_impl(attr, value)
}

#[proc_macro]
pub fn tool_group(attr: TokenStream) -> TokenStream {
    tool_group::tool_group_impl(attr)
}
