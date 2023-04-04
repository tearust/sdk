use tea_actorx2_core::worker_codec::OperationAbi;

use crate::{actor::Actor, error::Result, wasm::runtime::wasm_actor_entry};

use super::context::context;

#[inline(always)]
pub unsafe fn init() -> u32 {
	&mut context().abi as *mut OperationAbi as _
}

#[inline(always)]
pub unsafe fn init_handle(arg0: u32, arg1: u32) {
	context().abi.alloc_0(arg0 as _);
	context().abi.alloc_1(arg1 as _);
}

#[inline(always)]
pub unsafe fn handle<A>()
where
	A: Actor + Default,
{
	let input = context()
		.abi
		.unmarshal(|ptr, len| Ok(restore_vec(ptr, len)) as Result<_>)
		.unwrap();
	let output = wasm_actor_entry::<A>(input);
	context()
		.abi
		.marshal(output, |vec, ptr, len| {
			*len = vec.len() as _;
			*ptr = marshal_vec(vec);
			Ok(()) as Result<_>
		})
		.unwrap();
}

#[inline(always)]
pub unsafe fn finish_handle() {
	context().abi.dealloc()
}

#[inline(always)]
fn marshal_vec(vec: Vec<u8>) -> u32 {
	Box::into_raw(vec.into_boxed_slice()) as *mut u8 as _
}

#[inline(always)]
fn restore_vec(ptr: u32, len: u32) -> Vec<u8> {
	unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as _, len as _)) }.into_vec()
}

#[macro_export]
macro_rules! actor {
	($actor:ty) => {
		#[no_mangle]
		#[export_name = "abi_init"]
		pub unsafe extern "C" fn abi_init() -> u32 {
			$crate::abi::init()
		}

		#[no_mangle]
		#[export_name = "abi_init_handle"]
		pub unsafe extern "C" fn abi_init_handle(arg0: u32, arg1: u32) {
			$crate::abi::init_handle(arg0, arg1)
		}

		#[no_mangle]
		#[export_name = "abi_handle"]
		pub unsafe extern "C" fn abi_handle() {
			$crate::abi::handle::<$actor>()
		}

		#[no_mangle]
		#[export_name = "abi_finish_handle"]
		pub unsafe extern "C" fn abi_finish_handle() {
			$crate::abi::finish_handle()
		}
	};
}
