use std::{
	array::TryFromSliceError,
	borrow::Cow,
	char::{ParseCharError, TryFromCharError},
	net::AddrParseError,
	num::{ParseFloatError, ParseIntError, TryFromIntError},
	rc::Rc,
	str::{ParseBoolError, Utf8Error},
	string::FromUtf8Error,
	sync::Arc,
	time::SystemTimeError,
};

use hex::FromHexError;
use log::{ParseLevelError, SetLoggerError};
use thiserror::Error;

use crate::serde::error::UnexpectedType;

use super::{DescriptableMark, Descriptor};

tea_codec_macros::define_scope_internal! {
	Global {
		// Aggregate as v => Aggregate, "Multiple errors occurred", format!("{v:?}", ), v.0.iter().collect::<SmallVec<_>>();
		CannotBeNone => CannotBeNone, @Display, @Debug;
		UnexpectedType => UnexpectedType, @Display, @Debug;
		String as s => Unknown, format!("A string is thrown: \"{s}\""), s;
		&str as s => Unknown, format!("A string is thrown: \"{s}\""), *s;
		Box<str> as s => Unknown, format!("A string is thrown: \"{s}\""), **s;
		Rc<str> as s => Unknown, format!("A string is thrown: \"{s}\""), **s;
		Arc<str> as s => Unknown, format!("A string is thrown: \"{s}\""), **s;
		Cow<'_, str> as s => Unknown, format!("A string is thrown: \"{s}\""), **s;
		Box<dyn std::error::Error + '_> as e => Unknown, @Display, @Debug;
		Rc<dyn std::error::Error + '_> as e => Unknown, @Display, @Debug;
		Arc<dyn std::error::Error + '_> as e => Unknown, @Display, @Debug;
		serde_json::Error => JsonSerde, @Display, @Debug;
		bincode::Error => BincodeSerde, @Display, @Debug;
		Utf8Error => Utf8, @Display, @Debug;
		FromUtf8Error => Utf8, @Display, @Debug;
		std::io::Error => StdIo, @Display, @Debug;
		prost::EncodeError => ProstEncode, @Display, @Debug;
		prost::DecodeError => ProstDecode, @Display, @Debug;
		TryFromIntError => TryFrom, @Display, @Debug;
		TryFromCharError => TryFrom, @Display, @Debug;
		TryFromSliceError => TryFrom, @Display, @Debug;
		ParseBoolError => Parse, @Display, @Debug;
		ParseIntError => Parse, @Display, @Debug;
		ParseCharError => Parse, @Display, @Debug;
		ParseFloatError => Parse, @Display, @Debug;
		AddrParseError => Parse, @Display, @Debug;
		num_traits::ParseFloatError => Parse, @Display, @Debug;
		SystemTimeError => SystemTime, @Display, @Debug;
		ParseLevelError => Log, @Display, @Debug;
		SetLoggerError => Log, @Display, @Debug;
		base64::DecodeError => Base64Decode, @Display, @Debug;
		FromHexError => HexDecode, @Display, @Debug;
		std::sync::mpsc::RecvError => ChannelReceive, @Display, @Debug;
		crossbeam_channel::RecvError => ChannelReceive, @Display, @Debug;
		futures::channel::mpsc::TryRecvError => ChannelReceive, @Display, @Debug;
		futures::channel::oneshot::Canceled => ChannelReceive, @Display, @Debug;
		futures::channel::mpsc::SendError => ChannelSend, @Display, @Debug;
		RoutineTimeout => RoutineTimeout;
	}
}

macro_rules! define_send_error {
	(no_debug, $($t:tt)*) => {
		impl<T> DescriptableMark<$($t)*<T>> for Global {}

		impl<T> Descriptor<$($t)*<T>> for Global {
			fn name(_: &$($t)*<T>) -> Option<Cow<str>> {
				Some("ChannelSend".into())
			}

			fn summary(v: &$($t)*<T>) -> Option<Cow<str>> {
				Some(v.to_string().into())
			}
		}
	};
	($($t:tt)*) => {
		impl<T> DescriptableMark<$($t)*<T>> for Global {}

		impl<T> Descriptor<$($t)*<T>> for Global {
			fn name(_: &$($t)*<T>) -> Option<Cow<str>> {
				Some("ChannelSend".into())
			}

			fn summary(v: &$($t)*<T>) -> Option<Cow<str>> {
				Some(v.to_string().into())
			}

			fn detail(v: &$($t)*<T>) -> Option<Cow<str>> {
				Some(format!("{:?}", v).into())
			}
		}
	};
}

define_send_error!(std::sync::mpsc::SendError);
define_send_error!(crossbeam_channel::SendError);
define_send_error!(futures::channel::mpsc::TrySendError);

#[cfg(feature = "runtime")]
define_send_error!(no_debug, tokio::sync::mpsc::error::TrySendError);

#[cfg(feature = "runtime")]
define_send_error!(no_debug, tokio::sync::mpsc::error::SendError);

#[derive(Error, Debug, Default, PartialEq, Eq, Clone)]
#[error("Value \"{0}\" cannot be none")]
pub struct CannotBeNone(pub String);

#[derive(Error, Debug, Default, PartialEq, Eq, Clone)]
#[error("Bad binary format")]
pub struct BadBinaryFormat;

#[derive(Error, Debug, Default, PartialEq, Eq, Clone)]
#[error("Routine timeout at checkpoint {0}")]
pub struct RoutineTimeout(pub &'static str);
