use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Item};

#[cfg(feature = "tokio")]
#[proc_macro_attribute]
pub fn drive(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // You can parse _attr if you want to use the attribute argument (e.g., rservices)
    let input = parse_macro_input!(item as Item);

    // For now, just return the item unchanged
    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}

#[cfg(not(feature = "tokio"))]
#[proc_macro_attribute]
pub fn drive(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // You can parse _attr if you want to use the attribute argument (e.g., rservices)
    let input = parse_macro_input!(item as Item);

    // For now, just return the item unchanged
    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}