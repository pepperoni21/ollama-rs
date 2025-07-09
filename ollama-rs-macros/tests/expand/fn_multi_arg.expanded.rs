#[macro_use]
extern crate ollama_rs_macros;
#[doc(hidden)]
mod __hello_world_data {
    #[allow(unused_imports)]
    use super::*;
    use ollama_rs::re_exports::schemars;
    use ollama_rs::re_exports::serde;
    #[doc(hidden)]
    #[allow(non_camel_case_types, missing_docs)]
    #[serde(crate = "ollama_rs::re_exports::serde")]
    pub struct __hello_world__Params {
        ///The phrase to use for greeting
        pub greeting: String,
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
        "Say something"
    }
    async fn call(
        &mut self,
        Self::Params { greeting, name }: Self::Params,
    ) -> ::std::result::Result<
        ::std::string::String,
        ::std::boxed::Box<dyn ::std::error::Error + Send + Sync>,
    > {
        {
            Ok(
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("{0} {1}", greeting, name))
                }),
            )
        }
    }
}
#[doc(hidden)]
mod __dummy_data {
    #[allow(unused_imports)]
    use super::*;
    use ollama_rs::re_exports::schemars;
    use ollama_rs::re_exports::serde;
    #[doc(hidden)]
    #[allow(non_camel_case_types, missing_docs)]
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
        {
            Ok(
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("{0} {1} {2}", greeting, name, three),
                    )
                }),
            )
        }
    }
}
