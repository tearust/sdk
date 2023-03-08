use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, AttributeArgs, ItemFn, Meta, NestedMeta};

#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
	let mut args = parse_macro_input!(args as AttributeArgs);
	let item = parse_macro_input!(item as ItemFn);

	let init = match args.as_slice() {
		[NestedMeta::Meta(Meta::Path(init)), ..] => init.clone(),
		_ => {
			return syn::Error::new(
				item.span(),
				"Must specify an init function as the first argument.",
			)
			.to_compile_error()
			.into()
		}
	};

	args.remove(0);

	let ItemFn {
		attrs,
		vis,
		sig,
		block,
	} = item;

	quote! {
		#[::tea_sdk::third::tokio::test(#(#args),*)]
		#(#attrs)*
		#vis #sig {
			::tea_sdk::actorx::runtime::with_host(#init, async move #block).await
		}
	}
	.into()
}
