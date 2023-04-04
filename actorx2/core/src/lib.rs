#![feature(min_specialization)]
#![feature(const_trait_impl)]
#![feature(auto_traits)]
#![feature(negative_impls)]

extern crate tea_codec as tea_sdk;

pub mod actor;
pub mod error;
pub mod metadata;
#[cfg(feature = "sign")]
pub mod sign;

#[doc(hidden)]
pub mod worker_codec;

#[macro_export]
macro_rules! blocking {
	{let $capi:ident = &$capv:expr; $($e:tt)*} => {{
        let $capi = {
            fn capture<T>(v: &T) -> &'static T {
                unsafe { &*(v as *const T) }
            }
            capture(&$capv)
        };
		::tokio::task::spawn_blocking(move || { $($e)* }).await.unwrap()
	}};
	{let $capi:ident = &mut $capv:expr; $($e:tt)*} => {{
        let $capi = {
            fn capture_mut<T>(v: &mut T) -> &'static mut T {
                unsafe { &mut *(v as *mut T) }
            }
            capture_mut(&mut $capv)
        };
		::tokio::task::spawn_blocking(move || { $($e)* }).await.unwrap()
	}};
}

#[cfg(feature = "worker")]
pub mod wasm;
