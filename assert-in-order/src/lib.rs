use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parse, parse_macro_input, Error, Ident, Result, Token};

#[proc_macro]
pub fn in_order_init(input: TokenStream) -> TokenStream {
	let Init {category} = parse_macro_input!(input as Init);
	let ident = format_ident!("IN_ORDER_{}", category);
	let tokens = quote! {
		use dashmap::DashSet;
		use lazy_static::lazy_static;
		lazy_static! {
			#[allow(non_upper_case_globals)]
			static ref #ident: dashmap::DashSet<String> = DashSet::new();
		}
	};
	tokens.into()
}

struct Init {
	category: Ident,
}

impl Parse for Init {
	fn parse(input: syn::parse::ParseStream) -> Result<Self> {
		let category = input.parse()?;
		Ok(Init { category })
	}
}

// init!(test)
// in_order(test: prep)
// in_order(test: connect after prep)

/// Syntax:
/// `in_order(CATEGORY: THISLABEL before/after OTHERLABEL [before/after OTHERLABEL etc])`
#[proc_macro]
pub fn in_order(input: TokenStream) -> TokenStream {
	let InOrder {category, id, conditions} = parse_macro_input!(input as InOrder);
	let mut expanded: Vec<_> = Vec::new();
	let cat_ident = format_ident!("IN_ORDER_{}", category);
	for condition in conditions {
		expanded.push(match condition {
			Order::After(ident) => {
				let ident_str = ident.to_string();
				let assert_msg = format!("`{}` should come after `{}` in category `{}`", id, ident_str, category.to_string());
				quote! {
					assert!(#cat_ident.contains(#ident_str), #assert_msg);
				}
			}
			Order::Before(ident) => {
				let ident_str = ident.to_string();
				let assert_msg = format!("`{}` should come before `{}` in category `{}`", id, ident_str, category.to_string());
				quote! {
					assert!(!#cat_ident.contains(#ident_str), #assert_msg);
				}
			}
		})
	}
	let id_str = id.to_string();
	quote! {
		{
			#(#expanded)*
			let id_str = #id_str;
			let id_str = id_str.to_owned();
			assert!(#cat_ident.insert(id_str), "Identifier is not unique!");
		}
	}.into()
}

struct InOrder {
	category: Ident,
	id: Ident,
	conditions: Vec<Order>,
}

enum Order {
	Before(Ident),
	After(Ident),
}

impl Parse for InOrder {
	fn parse(input: syn::parse::ParseStream) -> Result<Self> {
		let category: Ident = input.parse()?;
		input.parse::<Token![:]>()?;
		let id: Ident = input.parse()?;

		let mut conditions = Vec::new();
		while !input.is_empty() {
			let before_or_after: Ident = input.parse()?;
			conditions.push(if before_or_after == format_ident!("after") {
				Order::After(input.parse::<Ident>()?)
			} else if before_or_after == format_ident!("before") {
				Order::Before(input.parse::<Ident>()?)
			} else {
				return Err(Error::new(
					input.span(),
					"Expected `before` or `after` followed by an identifier.",
				));
			});
		}

		Ok(InOrder {
			category,
			id,
			conditions,
		})
	}
}
