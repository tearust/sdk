//! tea-codec
//!
//! # About the Tea Project (teaproject.org)
//!
//! Tea Project (Trusted Execution & Attestation) is a Wasm runtime build on top of RoT(Root of Trust)
//! from both trusted hardware environment ,GPS, and blockchain technologies. Developer, Host and Consumer
//! do not have to trust any others to not only protecting privacy but also preventing cyber attacks.
//! The execution environment under remoted attestation can be verified by blockchain consensys.
//! Crypto economy is used as motivation that hosts are willing run trusted computing nodes.
//!

//!

#![feature(min_specialization)]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![allow(incomplete_features)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(type_alias_impl_trait)]
#![allow(clippy::module_inception)]

pub mod actor_txns;
pub mod runtime;
pub mod solc;
pub mod tapp;
#[cfg(feature = "vmh")]
pub mod vmh;

extern crate tea_codec as tea_sdk;
