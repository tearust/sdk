#![feature(drain_filter)]

use proc_macro::TokenStream;
mod errorx;
mod pricing;
mod serde;

use errorx::emit::{emit, emit_all};
use syn::parse_macro_input;

#[proc_macro]
pub fn define_scope_internal(input: TokenStream) -> TokenStream {
	let ast: errorx::ast::DefineScope = syn::parse(input).unwrap();
	emit::<true>(&ast).into()
}

#[proc_macro]
pub fn define_scope(input: TokenStream) -> TokenStream {
	let ast: errorx::ast::DefineScopes = syn::parse(input).unwrap();
	emit_all(&ast.0).into()
}

/// Marks the type with a generated unique type id that distinct among generics and package versions.
///
/// Use `#[response(<ResponseType>)]` to apply a response type so that it could be called on rpc calls.
///
/// If the type name ends with `Request`, then a response attribute is automatically added with the suffix of `Response` by convention.
///
/// # Examples
///
/// ```
/// use tea_codec_macros::TypeId;
///
/// #[derive(TypeId)]
/// // a `#[response(GetSystemTimeResponse)]` is automatically added by convention.
/// pub struct GetSystemTimeRequest;
///
/// #[derive(TypeId)]
/// pub struct GetSystemTimeResponse(pub u128);
/// ```
#[proc_macro_derive(TypeId, attributes(response))]
pub fn derive_type_id(input: TokenStream) -> TokenStream {
	let input: serde::ast::Input = parse_macro_input!(input);
	serde::emit::emit(input).into()
}

#[proc_macro_derive(Priced, attributes(price))]
pub fn derive_priced(input: TokenStream) -> TokenStream {
	let input: pricing::ast::Input = parse_macro_input!(input);
	pricing::emit::emit(input).into()
}
