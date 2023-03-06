use std::{any::type_name, fmt::Debug};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use tea_codec::{
	serde::{
		layout::{LayoutRead, LayoutWrite, UseFromToBytes, WithSize},
		ToBytes,
	},
	ResultExt,
};

use crate::{error::Result, ActorId};

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive)]
pub enum InputMessageKind {
	GuestCall,
	HostReturn,
	HostError,
}

pub type InputLayout = (
	u32,                             // Kind
	Option<u32>,                     //QuoteId
	Option<UseFromToBytes<ActorId>>, // Caller
	Vec<u8>,                         // Payload
);

pub fn encode_input(
	kind: InputMessageKind,
	quote_id: Option<usize>,
	caller: Option<ActorId>,
	payload: &[u8],
) -> Result<Vec<u8>> {
	let value = (
		kind as _,
		quote_id.map(|x| x as _),
		caller.as_ref(),
		payload,
	);
	let mut result = Vec::with_capacity(<InputLayout as LayoutWrite>::size(value)?);
	<InputLayout as LayoutWrite>::write(value, &mut result)?;
	Ok(result)
}

#[allow(clippy::type_complexity)]
pub fn decode_input(
	mut input: &[u8],
) -> Result<(InputMessageKind, Option<usize>, Option<ActorId>, &[u8])> {
	let (kind, quote_id, caller, payload) = <InputLayout as LayoutRead>::read(&mut input)?;
	Ok((
		FromPrimitive::from_u32(kind).ok_or_else(|| {
			tea_codec::serde::error::Error::from(tea_codec::serde::error::InvalidFormat(
				type_name::<InputMessageKind>(),
			))
		})?,
		quote_id.map(|x| x as _),
		caller,
		payload,
	))
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive)]
pub enum OutputMessageKind {
	HostCall,
	HostPost,
	GuestReturn,
	GuestError,
}

pub type OutputLayout<P = Vec<u8>> = WithSize<(
	u32,         // Kind
	Option<u32>, // QuoteId
	P,           // Payload
)>;

pub fn encode_output<P>(
	kind: OutputMessageKind,
	quote_id: Option<usize>,
	payload: P::Write<'_>,
) -> Result<Vec<u8>>
where
	P: LayoutWrite,
{
	let input = (kind as _, quote_id.map(|x| x as _), payload);
	let mut result = Vec::with_capacity(<OutputLayout<P> as LayoutWrite>::size(input)?);
	<OutputLayout<P> as LayoutWrite>::write(input, &mut result)?;
	Ok(result)
}

pub fn decode_output(mut output: &[u8]) -> Result<(OutputMessageKind, Option<usize>, &[u8])> {
	let (kind, quote_id, payload) = <OutputLayout as LayoutRead>::read(&mut output)?;
	Ok((
		FromPrimitive::from_u32(kind).ok_or_else(|| {
			tea_codec::serde::error::Error::from(tea_codec::serde::error::InvalidFormat(
				type_name::<InputMessageKind>(),
			))
		})?,
		quote_id.map(|x| x as _),
		payload,
	))
}

pub type InvokeLayout<T = Vec<u8>> = (UseFromToBytes<ActorId>, T);

pub type OutputInvokeLayout<T = Vec<u8>> = OutputLayout<WithSize<InvokeLayout<T>>>;

pub fn encode_invoke<T>(
	kind: OutputMessageKind,
	quote_id: Option<usize>,
	id: &ActorId,
	msg: &T,
) -> Result<Vec<u8>>
where
	T: ToBytes,
{
	let input = (kind as _, quote_id.map(|x| x as _), (id, msg));
	let mut result =
		Vec::with_capacity(<OutputInvokeLayout<UseFromToBytes<T>> as LayoutWrite>::size(input)?);
	<OutputInvokeLayout<UseFromToBytes<T>> as LayoutWrite>::write(input, &mut result)?;
	Ok(result)
}

pub fn decode_invoke(mut invoke: &[u8]) -> Result<(ActorId, &[u8])> {
	<InvokeLayout as LayoutRead>::read(&mut invoke).err_into()
}
