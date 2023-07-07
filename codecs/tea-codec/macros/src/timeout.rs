use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ItemFn};

pub fn emit_timeout_retry(
	ms: Expr,
	ItemFn {
		attrs,
		vis,
		sig,
		block,
	}: ItemFn,
	warner: TokenStream,
) -> TokenStream {
	let tag = sig.ident.to_string();
	quote! {
		#(#attrs)*
		#vis #sig
		{
			let mut i = 0;
			loop {
				match tea_sdk::Timeout::timeout(async {#block}, (#ms + i * 1000), concat!(#tag, " at ", file!(), " ", line!(), ":", column!())).await {
					Err(e) => {
						#warner("{}, retry {}", e, i);
						i += 1;
					}
					Ok(r) => break r,
				}
			}
		}
	}
}
