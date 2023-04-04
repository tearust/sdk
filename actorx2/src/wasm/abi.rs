use std::fmt::Arguments;

use tea_actorx2_core::worker_codec::OperationAbi;

use crate::{actor::Actor, error::Result, wasm::runtime::wasm_actor_entry};

use super::context::context;

#[inline(always)]
pub unsafe fn init() -> u32 {
	&mut context().abi as *mut OperationAbi as _
}

#[inline(always)]
pub unsafe fn init_handle(flag: u8, arg0: u32, arg1: u32) {
	context().abi.set_flag(flag);
	context().abi.alloc_0(arg0 as _);
	if flag == 0 {
		context().abi.alloc_1(arg1 as _);
	}
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
		#[export_name = "init"]
		pub unsafe extern "C" fn abi_init() -> u32 {
			$crate::abi::init()
		}

		#[no_mangle]
		#[export_name = "init_handle"]
		pub unsafe extern "C" fn abi_init_handle(flag: u8, arg0: u32, arg1: u32) {
			$crate::abi::init_handle(flag, arg0, arg1)
		}

		#[no_mangle]
		#[export_name = "handle"]
		pub unsafe extern "C" fn abi_handle() {
			$crate::abi::handle::<$actor>()
		}

		#[no_mangle]
		#[export_name = "finish_handle"]
		pub unsafe extern "C" fn abi_finish_handle() {
			$crate::abi::finish_handle()
		}
	};
}

extern "C" {
	#[link_name = "print"]
	#[allow(unused)]
	fn abi_print(ptr: *const u8, len: u32);
}

#[doc(hidden)]
//#[cfg(not(feature = "host"))]
pub fn _print(args: Arguments) {
	let string = std::fmt::format(args).into_bytes().into_boxed_slice();
	let len = string.len() as _;
	let ptr = string.as_ref() as *const [u8] as *const u8;
	unsafe {
		abi_print(ptr, len);
	}
}

// #[doc(hidden)]
// #[cfg(feature = "host")]
// pub fn _print(args: Arguments) {
// 	use std::io::Write;
// 	std::io::stdout().write_fmt(args).unwrap();
// }

#[macro_export]
macro_rules! print {
	($($arg: expr),*) => {
		$crate::abi::_print(format_args!($($arg),*));
	};
}

#[macro_export]
#[allow_internal_unstable(format_args_nl)]
macro_rules! println {
	($($arg: expr),*) => {
		$crate::abi::_print(format_args_nl!($($arg),*))
	};
}
