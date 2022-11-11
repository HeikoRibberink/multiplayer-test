use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, Data, DataStruct, DeriveInput, Error, Field, Ident, Result, Type};

pub fn net_bundle_derive_help(input: DeriveInput) -> Result<TokenStream> {
	let name = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let data: DataStruct = match input.data {
		Data::Struct(s) => s,
		Data::Enum(e) => {
			return Err(Error::new(
				e.enum_token.span,
				"Cannot derive NetBundle on an enum, only on a struct.",
			))
		}
		Data::Union(u) => {
			return Err(Error::new(
				u.union_token.span,
				"Cannot derive Netbundle on an union, only on a struct.",
			))
		}
	};

	let mut netw_fields_ident: Vec<Ident> = Vec::new();
	let mut netw_fields_ty: Vec<Type> = Vec::new();
	let mut netw_fields: Vec<Field> = Vec::new();

	// let mut fields_ident: Vec<Ident> = Vec::new();
	// let mut fields_ty: Vec<Type> = Vec::new();

	for field in data.fields.iter() {
		for attr in field.attrs.iter() {
			if attr.path.is_ident("networked") {
				// return Err(Error::new(field.ident.span(), "test"));
				netw_fields_ident.push(match field.ident.clone() {
					Some(ident) => ident,
					None => {
						return Err(Error::new(
							field.span(),
							"Tuple structs currently are not supported.",
						))
					}
				});
				netw_fields_ty.push(field.ty.clone());
				netw_fields.push(field.clone());
				break;
			}
		}
	}

	if netw_fields_ident.len() <= 0 {
		return Err(Error::new(
			name.span(),
			"Please mark at least one member with #[networked] to implement NetBundle.",
		));
	}

	// let get_fields_ident: Vec<Ident> = netw_fields_ident.iter().map(|i| {
	// 	format_ident!("self.{}", i)
	// }).collect();

	let net_comps_name = format_ident!("{}NetComps", name);

	Ok(quote! {
		#[derive(serde::Serialize, serde::Deserialize, Clone)]
		pub struct #net_comps_name {
			#(#netw_fields),*
		}
		impl #impl_generics NetBundle for #name #ty_generics #where_clause {
			// type NetComps = (#(#netw_fields_ty),*);
			type NetComps = #net_comps_name;
			fn get_networked(&self) -> Self::NetComps {
				// (
				// 	#(self.#netw_fields_ident),*
				// )
				#net_comps_name {
					#(#netw_fields_ident: self.#netw_fields_ident),*
				}
			}
			fn from_networked(components: Self::NetComps) -> Self {
				let #net_comps_name { #(#netw_fields_ident),* } = components;
				Self {
					#(#netw_fields_ident),*,
					..Default::default()
				}
			}
		}
	}
	.into())
}
