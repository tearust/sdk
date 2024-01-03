use proc_macro::TokenStream;
mod handle;
mod pricing;
mod serde;
mod timeout;

use quote::{quote, ToTokens};
use syn::parse_macro_input;

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

#[proc_macro_attribute]
/// Impls `Handle` traits for the type, making it available for handling requests based on the serde system.
///
/// You can only use this once per type, for it must covert all request types that it handles.
///
/// Every handle functions have to be `async`, not `pub`, named `handle`, with `&self` receiver, otherwise it would be normal associated functions of such type.
///
/// # Examples
///
/// ```
/// struct Actor;
///
/// #[handles]
/// impl Actor {
/// 	async fn handle(&self, _: Activate) -> Result<_> {
/// 		println!("Activate!");
/// 		Ok(())
/// 	}
///
/// 	async fn handle(&self, GreetingsRequest(name): _) -> Result<_> {
/// 		println!("Hello {name}.");
/// 		Ok(())
/// 	}
/// }
/// ```
pub fn handles(_args: TokenStream, input: TokenStream) -> TokenStream {
	let input: handle::HandlesImpl = parse_macro_input!(input);
	input.to_token_stream().into()
}

#[proc_macro_attribute]
pub fn timeout_retry(args: TokenStream, input: TokenStream) -> TokenStream {
	let ms = parse_macro_input!(args);
	let f = parse_macro_input!(input);
	timeout::emit_timeout_retry(ms, f, quote! {tracing::warn!}).into()
}

#[proc_macro_attribute]
pub fn timeout_retry_worker(args: TokenStream, input: TokenStream) -> TokenStream {
	let ms = parse_macro_input!(args);
	let f = parse_macro_input!(input);
	timeout::emit_timeout_retry(ms, f, quote! {println!}).into()
}
