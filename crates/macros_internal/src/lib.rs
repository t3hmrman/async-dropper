extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn test_async_runtime(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    quote! {
        #[cfg_attr(feature = "tokio", tokio::test(flavor = "multi_thread"))]
        #[cfg_attr(feature = "async-std", async_std::test)]
        #input_fn
    }
    .into()
}
