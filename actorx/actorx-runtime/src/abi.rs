use std::fmt::Arguments;
#[cfg(any(not(feature = "mock"), feature = "no-mock"))]
use std::ptr::slice_from_raw_parts_mut;

use lazy_static::lazy_static;
use tokio::runtime::Runtime;

#[macro_export]
#[cfg(any(not(feature = "mock"), feature = "no-mock"))]
macro_rules! actor {
	($actor: ty) => {
		$crate::actor!($crate::NoCallingCxWrapper<$actor>, with_caller);
	};
	($actor: ty, with_caller) => {
		#[no_mangle]
		#[export_name = "init_handle"]
		pub unsafe extern "C" fn abi_init_handle(len: u32) -> u32 {
			let mut result = ::std::vec::Vec::<u8>::with_capacity(len as _);
			result.set_len(len as _);
			::std::boxed::Box::into_raw(result.into_boxed_slice()) as *mut u8 as _
		}

		#[no_mangle]
		#[export_name = "handle"]
		pub unsafe extern "C" fn abi_handle(ptr: u32, len: u32) -> u32 {
			let input = ::std::boxed::Box::from_raw(::std::ptr::slice_from_raw_parts_mut(
				ptr as _, len as _,
			))
			.into_vec();
			let output = $crate::RUNTIME.block_on($crate::handle::<$actor>(input));
			::std::boxed::Box::into_raw(output.into_boxed_slice()) as *mut u8 as _
		}

		#[no_mangle]
		#[export_name = "finish_handle"]
		pub unsafe extern "C" fn abi_finish_handle(ptr: u32, len: u32) {
			drop(::std::boxed::Box::from_raw(
				::std::ptr::slice_from_raw_parts_mut(ptr as *mut u8, len as _),
			));
		}
	};
}

#[macro_export]
#[cfg(all(not(feature = "no-mock"), feature = "mock"))]
macro_rules! actor {
	($actor: ty, with_caller) => {};
	($actor: ty) => {};
}

#[cfg(any(not(feature = "mock"), feature = "no-mock"))]
extern "C" {
	#[link_name = "print"]
	fn abi_print(ptr: *const u8, len: u32);
}

#[no_mangle]
#[export_name = "finish_print"]
#[cfg(any(not(feature = "mock"), feature = "no-mock"))]
pub unsafe extern "C" fn abi_finish_print(ptr: *mut u8, len: u32) {
	drop(Box::from_raw(slice_from_raw_parts_mut(ptr, len as _)))
}

#[doc(hidden)]
#[cfg(any(not(feature = "mock"), feature = "no-mock"))]
pub fn _print(args: Arguments) {
	let string = std::fmt::format(args).into_bytes().into_boxed_slice();
	let len = string.len() as _;
	let ptr = string.as_ref() as *const [u8] as *const u8;
	unsafe {
		abi_print(ptr, len);
	}
}

#[doc(hidden)]
#[cfg(all(not(feature = "no-mock"), feature = "mock"))]
pub fn _print(args: Arguments) {
	use std::io::Write;
	std::io::stdout().write_fmt(args).unwrap();
}

#[cfg(any(not(feature = "mock"), feature = "no-mock"))]
pub fn print_bytes(bytes: &[u8]) {
	let len = bytes.len() as _;
	let ptr = bytes as *const [u8] as *const u8;
	unsafe {
		abi_print(ptr, len);
	}
}

#[cfg(all(not(feature = "no-mock"), feature = "mock"))]
pub fn print_bytes(bytes: &[u8]) {
	let s = String::from_utf8_lossy(bytes);
	print!("{s}")
}

#[macro_export]
macro_rules! print {
	($($arg: expr),*) => {
		$crate::_print(format_args!($($arg),*));
	};
}

#[macro_export]
#[allow_internal_unstable(format_args_nl)]
macro_rules! println {
	($($arg: expr),*) => {
		$crate::_print(format_args_nl!($($arg),*));
	};
}

lazy_static! {
	#[doc(hidden)]
	pub static ref RUNTIME: Runtime = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap();
}
