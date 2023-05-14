pub mod actor;
pub mod error;
pub mod metadata;
#[cfg(feature = "sign")]
pub mod sign;

#[doc(hidden)]
#[cfg(any(feature = "worker", feature = "host", feature = "wasm"))]
pub mod worker_codec;

#[doc(hidden)]
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
