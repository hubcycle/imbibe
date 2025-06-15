use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, Ident, Token, parse_macro_input};

struct SignerFields {
	fields: Vec<Ident>,
}

#[proc_macro_derive(GetSigners, attributes(signer_fields))]
pub fn derive_get_signers(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let struct_name = &input.ident;

	let signer_fields = input
		.attrs
		.iter()
		.find_map(|attr| {
			attr.path()
				.is_ident("signer_fields")
				.then(|| attr.parse_args::<SignerFields>().ok().map(|sf| sf.fields))
				.flatten()
		})
		.unwrap_or_default();

	let mut borrowed_field_exprs = vec![];
	let mut owned_field_exprs = vec![];
	if let syn::Data::Struct(data_struct) = &input.data {
		for field_name in signer_fields {
			let field = data_struct
				.fields
				.iter()
				.find(|f| f.ident.as_ref().map(|id| id == &field_name).unwrap_or(false));

			if let Some(field) = field {
				let field_ident = field.ident.as_ref().unwrap();
				let ty = &field.ty;
				let ty_str = quote! { #ty }.to_string().replace(' ', "");

				let (borrowed_field_expr, owned_field_expr) = match ty_str.as_str() {
					"::prost::alloc::string::String" => (
						quote! { .chain(std::iter::once(self.#field_ident.as_str())) },
						quote! { .chain(std::iter::once(self.#field_ident)) },
					),
					"::prost::alloc::vec::Vec<::prost::alloc::string::String>" => (
						quote! { .chain(self.#field_ident.iter().map(|s| s.as_str())) },
						quote! { .chain(self.#field_ident) },
					),
					s if s.starts_with("::prost::alloc::vec::Vec<") => (
						quote! { .chain(self.#field_ident.iter().flat_map(GetSigners::signers)) },
						quote! { .chain(self.#field_ident.into_iter().flat_map(GetSigners::signers)) },
					),
					_ => (
						quote! { .chain(GetSigners::singers(&self.#field_ident)) },
						quote! { .chain(GetSigners::singers(self.#field_ident)) },
					),
				};

				borrowed_field_exprs.push(borrowed_field_expr);
				owned_field_exprs.push(owned_field_expr);
			}
		}
	}

	let expanded = quote! {
		impl<'a> GetSigners for &'a #struct_name {
			type Signer = &'a str;

			fn signers(self) -> impl Iterator<Item = Self::Signer> {
				std::iter::empty()
					#(#borrowed_field_exprs)*
			}
		}


		impl GetSigners for #struct_name {
			type Signer = String;

			fn signers(self) -> impl Iterator<Item = Self::Signer> {
				std::iter::empty()
					#(#owned_field_exprs)*
			}
		}
	};

	TokenStream::from(expanded)
}

impl Parse for SignerFields {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut fields = vec![];
		while !input.is_empty() {
			fields.push(input.parse()?);
			if input.peek(Token![,]) {
				input.parse::<Token![,]>()?;
			}
		}

		Ok(SignerFields { fields })
	}
}
