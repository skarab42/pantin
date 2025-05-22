#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::missing_errors_doc)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(WebDriverCommand)]
pub fn derive_web_driver_command(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let parameters = format_ident!("{}Parameters", ident);
    let response = format_ident!("{}Response", ident);
    let command = format!("WebDriver:{ident}");

    let expanded = quote! {
        impl #ident {
            #[must_use]
            pub const fn new(parameters: #parameters) -> Self {
                Self { parameters }
            }
        }

        impl Command for #ident {
            type Parameters = #parameters;
            type Response = #response;

            fn name(&self) -> &'static str {
                #command
            }

            fn parameters(&self) -> &Self::Parameters {
                &self.parameters
            }
        }

    };

    expanded.into()
}
