use proc_macro2::Ident;
use syn::{
	parse::{Parse, ParseStream},
	spanned::Spanned,
	Attribute, DeriveInput, Generics, Result, Type, TypePath,
};

pub struct Input {
	pub ident: Ident,
	pub generics: Generics,
	pub resp: Option<Type>,
}

const ATTR_RESPONSE_IDENT: &str = "response";

impl Parse for Input {
	fn parse(input: ParseStream) -> Result<Self> {
		let body = DeriveInput::parse(input)?;
		let mut resp = None;
		for attr in &body.attrs {
			if is_single_ident(ATTR_RESPONSE_IDENT)(attr) {
				if resp.is_some() {
					return Err(syn::Error::new(
						attr.span(),
						"Multiple response type specified.",
					));
				}
				resp = Some(attr.parse_args()?);
			}
		}
		if resp.is_none() {
			let ident = body.ident.to_string();
			if let Some(ident) = ident.strip_suffix("Request") {
				resp = Some(Type::Path(TypePath {
					qself: None,
					path: Ident::new((ident.to_string() + "Response").as_str(), body.ident.span())
						.into(),
				}))
			}
		}
		Ok(Self {
			resp,
			ident: body.ident,
			generics: body.generics,
		})
	}
}

const fn is_single_ident(cmp: &str) -> impl Fn(&Attribute) -> bool + '_ {
	move |attr| {
		if let Some(ident) = attr.path.get_ident() {
			ident == cmp
		} else {
			false
		}
	}
}
