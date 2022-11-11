use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod net_bundle;

#[proc_macro_derive(NetBundle, attributes(networked))]
pub fn net_bundle_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	net_bundle::net_bundle_derive_help(input).unwrap_or_else(|r| r.to_compile_error().into())
}
