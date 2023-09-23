use proc_macro::TokenStream;

extern crate quote;
extern crate syn;

#[cfg(feature = "derive")]
mod task;

#[cfg(feature = "derive")]
#[proc_macro_derive(Task, attributes(attr))]
pub fn derive_task(input: TokenStream) -> TokenStream {
    use crate::task::parse_task;
    use syn::{parse_macro_input, DeriveInput};

    let input = parse_macro_input!(input as DeriveInput);
    let token = parse_task(&input);
    TokenStream::from(token)
}