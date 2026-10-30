use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use super::{extract_docs, function_impl};

fn expand(input: TokenStream) -> String {
    let output = function_impl(TokenStream::new(), input);
    let file: syn::File = syn::parse2(output).expect("macro output should parse as a file");
    prettyplease::unparse(&file)
}

fn parse_item(input: TokenStream) -> ItemFn {
    syn::parse2(input).expect("test input should parse as an item function")
}

#[test]
fn expands_function_without_arguments() {
    let output = expand(quote! {
        /// Say hello
        async fn hello_world() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok("Hello".to_string())
        }
    });

    let expected = r#"#[doc(hidden)]
mod __hello_world_data {
    #[allow(unused_imports)]
    use super::*;
    use ollama_rs::re_exports::schemars;
    use ollama_rs::re_exports::serde;
    #[doc(hidden)]
    #[allow(non_camel_case_types, missing_docs)]
    #[derive(serde::Deserialize, schemars::JsonSchema)]
    #[serde(crate = "ollama_rs::re_exports::serde")]
    pub struct __hello_world__Params {}
}
#[allow(non_camel_case_types)]
struct hello_world;
impl ::ollama_rs::generation::tools::Tool for hello_world {
    type Params = __hello_world_data::__hello_world__Params;
    #[inline]
    fn name() -> &'static str {
        "hello_world"
    }
    #[inline]
    fn description() -> &'static str {
        "Say hello"
    }
    async fn call(
        &mut self,
        Self::Params {}: Self::Params,
    ) -> ::std::result::Result<
        ::std::string::String,
        ::std::boxed::Box<dyn ::std::error::Error + Send + Sync>,
    > {
        { Ok("Hello".to_string()) }
    }
}
"#;

    assert_eq!(output, expected);
}

#[test]
fn expands_function_with_one_argument() {
    let output = expand(quote! {
        /// Say hello
        ///
        /// * name - Whom to say hello to
        async fn hello_world(name: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok(format!("Hello {}", name))
        }
    });

    let expected = r#"#[doc(hidden)]
mod __hello_world_data {
    #[allow(unused_imports)]
    use super::*;
    use ollama_rs::re_exports::schemars;
    use ollama_rs::re_exports::serde;
    #[doc(hidden)]
    #[allow(non_camel_case_types, missing_docs)]
    #[derive(serde::Deserialize, schemars::JsonSchema)]
    #[serde(crate = "ollama_rs::re_exports::serde")]
    pub struct __hello_world__Params {
        ///Whom to say hello to
        pub name: String,
    }
}
#[allow(non_camel_case_types)]
struct hello_world;
impl ::ollama_rs::generation::tools::Tool for hello_world {
    type Params = __hello_world_data::__hello_world__Params;
    #[inline]
    fn name() -> &'static str {
        "hello_world"
    }
    #[inline]
    fn description() -> &'static str {
        "Say hello"
    }
    async fn call(
        &mut self,
        Self::Params { name }: Self::Params,
    ) -> ::std::result::Result<
        ::std::string::String,
        ::std::boxed::Box<dyn ::std::error::Error + Send + Sync>,
    > {
        { Ok(format!("Hello {}", name)) }
    }
}
"#;

    assert_eq!(output, expected);
}

#[test]
fn expands_function_with_multiple_arguments() {
    let output = expand(quote! {
        /// Dummy
        ///
        /// * one - Arg one
        /// * two - Arg two
        /// * three - Arg three
        async fn dummy(
            one: String,
            two: i32,
            three: bool,
        ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok(format!("{} {} {}", one, two, three))
        }
    });

    let expected = r#"#[doc(hidden)]
mod __dummy_data {
    #[allow(unused_imports)]
    use super::*;
    use ollama_rs::re_exports::schemars;
    use ollama_rs::re_exports::serde;
    #[doc(hidden)]
    #[allow(non_camel_case_types, missing_docs)]
    #[derive(serde::Deserialize, schemars::JsonSchema)]
    #[serde(crate = "ollama_rs::re_exports::serde")]
    pub struct __dummy__Params {
        ///Arg one
        pub one: String,
        ///Arg two
        pub two: i32,
        ///Arg three
        pub three: bool,
    }
}
#[allow(non_camel_case_types)]
struct dummy;
impl ::ollama_rs::generation::tools::Tool for dummy {
    type Params = __dummy_data::__dummy__Params;
    #[inline]
    fn name() -> &'static str {
        "dummy"
    }
    #[inline]
    fn description() -> &'static str {
        "Dummy"
    }
    async fn call(
        &mut self,
        Self::Params { one, two, three }: Self::Params,
    ) -> ::std::result::Result<
        ::std::string::String,
        ::std::boxed::Box<dyn ::std::error::Error + Send + Sync>,
    > {
        { Ok(format!("{} {} {}", one, two, three)) }
    }
}
"#;

    assert_eq!(output, expected);
}

#[test]
fn extracts_description_and_parameter_docs() {
    let input = parse_item(quote! {
        /// Say something
        ///
        /// * greeting - The phrase to use for greeting
        /// * name - Whom to say hello to
        async fn hello_world(
            greeting: String,
            name: String,
        ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok(format!("{} {}", greeting, name))
        }
    });

    let docs = extract_docs(&input).expect("function should have docs");

    assert_eq!(docs.description, "Say something");
    assert_eq!(
        docs.parameter_docs.get("greeting").map(String::as_str),
        Some("The phrase to use for greeting")
    );
    assert_eq!(
        docs.parameter_docs.get("name").map(String::as_str),
        Some("Whom to say hello to")
    );
}

#[test]
fn requires_async() {
    let output = expand(quote! {
        /// Say hello
        fn hello_world() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok("Hello".to_string())
        }
    });

    assert!(output.contains("function must be async"));
}

#[test]
fn requires_documentation() {
    let output = expand(quote! {
        async fn hello_world() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok("Hello".to_string())
        }
    });

    assert!(output.contains("function must be documented"));
}

#[test]
fn rejects_generics() {
    let output = expand(quote! {
        /// Say hello
        async fn hello_world<T>() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok("Hello".to_string())
        }
    });

    assert!(output.contains("functions with generics are not supported"));
}

#[test]
fn rejects_self_argument() {
    let output = expand(quote! {
        /// Say hello
        async fn hello_world(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok("Hello".to_string())
        }
    });

    assert!(output.contains("self argument is not allowed"));
}

#[test]
fn rejects_non_identifier_argument_patterns() {
    let output = expand(quote! {
        /// Say hello
        async fn hello_world((a, b): (i32, i32)) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok(format!("{} {}", a, b))
        }
    });

    assert!(output.contains("only named arguments are allowed"));
}
