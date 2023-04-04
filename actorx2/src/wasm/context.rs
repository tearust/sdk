use std::{any::Any, cell::UnsafeCell, pin::Pin};

use tea_actorx2_core::worker_codec::OperationAbi;

use crate::{context::CallingStack, error::Result};

static CONTEXT: ContextCell = ContextCell(UnsafeCell::new(Context {
	calling_stack: None,
	input: None,
	output: None,
	execution: None,
	abi: OperationAbi::default(),
}));

struct ContextCell(UnsafeCell<Context>);
unsafe impl Sync for ContextCell {}

pub struct Context {
	pub calling_stack: Option<CallingStack>,
	pub input: Option<Result<Vec<u8>>>,
	pub output: Option<(Vec<u8>, Vec<u8>)>,
	pub execution: Option<Pin<Box<dyn Any>>>,
	pub abi: OperationAbi,
}

#[inline(always)]
pub fn context() -> &'static mut Context {
	unsafe { &mut *CONTEXT.0.get() }
}
