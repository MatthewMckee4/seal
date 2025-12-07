mod options_metadata;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(OptionsMetadata, attributes(field, doc, option_group))]
pub fn derive_options_metadata(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    options_metadata::derive_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
