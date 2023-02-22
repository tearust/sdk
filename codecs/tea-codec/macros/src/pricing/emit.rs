use proc_macro2::TokenStream;
use quote::quote;
use syn::{ConstParam, GenericParam, Generics, LifetimeDef, TypeParam};

use super::ast::Input;

pub fn emit(
    Input {
        ident,
        generics,
        expr,
    }: Input,
) -> TokenStream {
    let expr = if let Some(expr) = expr {
        quote! { #expr }
    } else {
        quote! { None }
    };

    let Generics {
        lt_token,
        params,
        gt_token,
        where_clause,
    } = generics;

    let params_idents = params.iter().map(|x| match x {
        GenericParam::Type(TypeParam { ident, .. }) => quote! { #ident },
        GenericParam::Lifetime(LifetimeDef { lifetime, .. }) => quote! { #lifetime },
        GenericParam::Const(ConstParam { ident, .. }) => quote! { #ident },
    });

    let params_idents = if params_idents.len() != 0 {
        quote! { <#(#params_idents),*> }
    } else {
        quote! {}
    };

    quote! {
        impl #lt_token #params #gt_token ::tea_sdk::pricing::Priced for #ident #params_idents
            #where_clause
        {
            fn price(&self) -> Option<u64> {
                (#expr).into()
            }
        }
    }
}
