use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote;
use syn::{
	punctuated::Punctuated, Path, PredicateType, TraitBound, TraitBoundModifier, Type,
	TypeParamBound, TypePath, WherePredicate,
};

use super::ast::Input;

pub fn emit(
	Input {
		ident,
		mut generics,
		resp,
	}: Input,
) -> TokenStream {
	let type_name = Literal::string(ident.to_string().as_str());
	let targs = generics
		.type_params()
		.map(|x| x.ident.clone())
		.collect::<Vec<_>>();
	let wc = generics.make_where_clause();
	for t in &targs {
		wc.predicates.push(WherePredicate::Type(PredicateType {
			lifetimes: None,
			bounded_ty: Type::Path(TypePath {
				qself: None,
				path: t.clone().into(),
			}),
			colon_token: Default::default(),
			bounds: {
				let mut segments = Punctuated::new();
				segments.push(TypeParamBound::Trait(TraitBound {
					paren_token: None,
					modifier: TraitBoundModifier::None,
					lifetimes: None,
					path: Path {
						leading_colon: Some(Default::default()),
						segments: {
							let mut segments = Punctuated::new();
							for seg in ["tea_codec", "serde", "TypeId"] {
								segments.push(Ident::new(seg, ident.span()).into());
							}
							segments
						},
					},
				}));
				segments
			},
		}));
	}

	let mut value = quote! { ::tea_sdk::const_concat::ConstStr::empty().append_str(concat!(module_path!(), "::", #type_name, "@", env!("CARGO_PKG_VERSION"))) };
	if !targs.is_empty() {
		value = quote! { #value.append_str("<") };
		for (i, t) in targs.iter().enumerate() {
			if i == 0 {
				value = quote! { #value.append_str(<#t as ::tea_sdk::serde::TypeId>::TYPE_ID) };
			} else {
				value = quote! { #value.append_str(",").append_str(<#t as ::tea_sdk::serde::TypeId>::TYPE_ID) };
			}
		}
		value = quote! { #value.append_str(">") };
	}

	let impl_type_id = quote! {
		impl<#(#targs),*> ::tea_sdk::serde::TypeIdBuf for #ident<#(#targs),*>
			#wc
		{
			const TYPE_ID_BUF: ::tea_sdk::const_concat::ConstStr = #value;
		}

		impl<#(#targs),*> ::tea_sdk::serde::TypeId for #ident<#(#targs),*>
			#wc
		{
			const TYPE_ID: &'static str = <Self as ::tea_sdk::serde::TypeIdBuf>::TYPE_ID_BUF.as_str();
		}
	};

	let resp = if let Some(resp) = resp {
		quote! {
			impl<#(#targs),*> ::tea_sdk::serde::handle::Request for #ident<#(#targs),*>
			{
				type Response = #resp;
			}
		}
	} else {
		quote! {}
	};

	quote! {
		#impl_type_id
		#resp
	}
}
