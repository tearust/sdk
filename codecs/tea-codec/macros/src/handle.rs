use proc_macro2::Span;
use quote::ToTokens;
use syn::{
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
	spanned::Spanned,
	token::Colon2,
	AngleBracketedGenericArguments, Error, FnArg, GenericArgument, Generics, Ident, ImplItem,
	ImplItemMethod, ImplItemType, ItemImpl, Pat, PatStruct, PatTupleStruct, PatType, Path,
	PathArguments, PathSegment, Result, ReturnType, Token, Type, TypePath, Visibility,
};

pub struct HandlesImpl {
	pub impls: Vec<ItemImpl>,
}

impl TryFrom<ItemImpl> for HandlesImpl {
	type Error = Error;
	fn try_from(mut raw_impl: ItemImpl) -> Result<Self> {
		let mut impls = Vec::new();
		let mut reqs = None;
		let mut raw_items = Vec::new();

		let items = raw_impl.items;
		raw_impl.items = Vec::new();
		let impl_template = raw_impl;

		for item in items {
			match item {
				ImplItem::Method(method) if method.sig.ident == "handle" => {
					let HandleImpl { req: ty, method } = method.try_into()?;
					reqs = Some({
						let others = if let Some(reqs) = reqs {
							reqs
						} else {
							Type::Path(TypePath {
								qself: None,
								path: path_handle(Span::call_site(), |seg| {
									seg.push(PathSegment {
										ident: Ident::new("Fail", Span::call_site()),
										arguments: PathArguments::None,
									});
								}),
							})
						};
						Type::Path(TypePath {
							qself: None,
							path: path_handle(ty.span(), |seg| {
								seg.push(path_seg(ty.span(), "With", |args| {
									args.push(GenericArgument::Type(Type::clone(&ty)));
									args.push(GenericArgument::Type(others));
								}))
							}),
						})
					});

					let mut impl_ = impl_template.clone();
					impl_.items.push(ImplItem::Method(method));

					impl_.trait_ = Some((
						None,
						path_handle(impl_.self_ty.span(), |seg| {
							seg.push(path_seg_single(impl_.self_ty.span(), "Handle", *ty))
						}),
						Token![for](impl_.self_ty.span()),
					));

					impls.push(impl_);
				}
				raw => raw_items.push(raw),
			}
		}

		if let Some(reqs) = reqs {
			let mut impl_ = impl_template.clone();
			impl_.items.push(ImplItem::Type(ImplItemType {
				attrs: Vec::new(),
				vis: Visibility::Inherited,
				defaultness: None,
				type_token: Token![type](reqs.span()),
				ident: Ident::new("List", reqs.span()),
				generics: Generics::default(),
				eq_token: Token![=](reqs.span()),
				semi_token: Token![;](reqs.span()),
				ty: reqs,
			}));
			impl_.trait_ = Some((
				None,
				path_handle(impl_.self_ty.span(), |seg| {
					seg.push(PathSegment {
						ident: Ident::new("Handles", impl_.self_ty.span()),
						arguments: PathArguments::None,
					})
				}),
				Token![for](impl_.self_ty.span()),
			));
			impls.push(impl_);
		}

		if !raw_items.is_empty() {
			let mut raw_impl = impl_template;
			raw_impl.items = raw_items;
			impls.push(raw_impl);
		}

		Ok(Self { impls })
	}
}

impl Parse for HandlesImpl {
	fn parse(input: ParseStream) -> Result<Self> {
		input.parse::<ItemImpl>()?.try_into()
	}
}

impl ToTokens for HandlesImpl {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		for item in &self.impls {
			item.to_tokens(tokens);
		}
	}
}

struct HandleImpl {
	req: Box<Type>,
	method: ImplItemMethod,
}

impl TryFrom<ImplItemMethod> for HandleImpl {
	type Error = Error;
	fn try_from(
		ImplItemMethod {
			attrs,
			mut sig,
			block,
			vis,
			defaultness,
		}: ImplItemMethod,
	) -> Result<Self> {
		let sig_span = sig.span();

		let req = sig
			.inputs
			.iter_mut()
			.nth(1)
			.ok_or_else(|| Error::new(sig_span, "An request parameter is required."))?;
		let FnArg::Typed(PatType{pat,ty: req,..}) = req else {
            return Err(Error::new(req.span(), "The request parameter must not be a self receiver."));
        };

		if let Type::Infer(infer) = &mut **req {
			let (Pat::Struct(PatStruct{ path,.. }) | Pat::TupleStruct(PatTupleStruct{path,..})) = &mut **pat else {
                return Err(Error::new(infer.span(), "The request parameter must have a struct pattern to infer a proper type."));
            };

			**req = Type::Path(TypePath {
				qself: None,
				path: path.clone(),
			});
		}

		if let ReturnType::Type(_, ty) = &mut sig.output {
			if let Type::Path(TypePath {
				path: Path { segments, .. },
				..
			}) = ty.as_mut()
			{
				if let Some(PathSegment {
					arguments:
						PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
					..
				}) = segments.last_mut()
				{
					if let Some(GenericArgument::Type(ty @ Type::Infer(_))) = args.first_mut() {
						*ty = Type::Path(TypePath {
							qself: Some(syn::QSelf {
								lt_token: Token![<](ty.span()),
								ty: req.clone(),
								position: 4,
								as_token: Some(Token![as](ty.span())),
								gt_token: Token![>](ty.span()),
							}),
							path: path_handle(ty.span(), |seg| {
								seg.push(PathSegment {
									ident: Ident::new("Request", ty.span()),
									arguments: PathArguments::None,
								});
								seg.push(PathSegment {
									ident: Ident::new("Response", ty.span()),
									arguments: PathArguments::None,
								});
							}),
						});
					}
				}
			}
		}

		let req = req.clone();

		Ok(Self {
			req,
			method: ImplItemMethod {
				attrs,
				vis,
				defaultness,
				sig,
				block,
			},
		})
	}
}

fn path_seg(
	span: Span,
	id: &str,
	args: impl FnOnce(&mut Punctuated<GenericArgument, Token![,]>),
) -> PathSegment {
	PathSegment {
		ident: Ident::new(id, span),
		arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
			colon2_token: None,
			lt_token: Token![<](span),
			args: {
				let mut arguments = Punctuated::new();
				args(&mut arguments);
				arguments
			},
			gt_token: Token![>](span),
		}),
	}
}
fn path_seg_single(span: Span, id: &str, t_arg: Type) -> PathSegment {
	path_seg(span, id, |args| args.push(GenericArgument::Type(t_arg)))
}

fn path_handle(span: Span, seg: impl FnOnce(&mut Punctuated<PathSegment, Colon2>)) -> Path {
	Path {
		leading_colon: Some(Token![::](span)),
		segments: {
			let mut segments = Punctuated::new();
			segments.push(PathSegment {
				ident: Ident::new("tea_sdk", span),
				arguments: PathArguments::None,
			});
			segments.push(PathSegment {
				ident: Ident::new("serde", span),
				arguments: PathArguments::None,
			});
			segments.push(PathSegment {
				ident: Ident::new("handle", span),
				arguments: PathArguments::None,
			});
			seg(&mut segments);
			segments
		},
	}
}
