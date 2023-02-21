use syn::{
	parse::{Parse, ParseStream},
	spanned::Spanned,
	DeriveInput, Error, Expr, Generics, Ident, Result,
};

pub const ATTR_PRICE_IDENT: &str = "price";

pub struct Input {
	pub ident: Ident,
	pub generics: Generics,
	pub expr: Option<Expr>,
}

impl Parse for Input {
	fn parse(input: ParseStream) -> Result<Self> {
		let DeriveInput {
			ident,
			generics,
			attrs,
			..
		} = DeriveInput::parse(input)?;

		let attr = {
			let mut it = attrs.iter().filter(|x| x.path.is_ident(ATTR_PRICE_IDENT));
			let attr = it.next();
			if let Some(next) = it.next() {
				return Err(Error::new(
					next.span(),
					"Only one pricing function is allowed.",
				));
			}
			attr
		};

		let expr = attr.map(|attr| attr.parse_args()).transpose()?;

		Ok(Self {
			ident,
			generics,
			expr,
		})
	}
}
