use std::{any::type_name, mem::size_of};

use impl_trait_for_tuples::impl_for_tuples;

use crate::ResultExt;

use super::{
	error::{InvalidFormat, Result},
	FromBytes, SerBuf, ToBytes,
};

pub trait LayoutWrite: Sized {
	type Write<'a>: Copy
	where
		Self: 'a;
	fn size(value: Self::Write<'_>) -> Result<usize>;
	fn write(value: Self::Write<'_>, buf: impl SerBuf) -> Result<()>;
}

pub trait LayoutRead: Sized {
	type Read<'a>;
	fn read<'a>(buf: &mut &'a [u8]) -> Result<Self::Read<'a>>;
}

macro_rules! impl_nums {
	($($t:ty),*) => {$(
		impl LayoutWrite for $t {
			type Write<'a> = Self;

			#[inline(always)]
			fn size(_: Self::Write<'_>) -> Result<usize> {
				Ok(size_of::<Self>())
			}

			#[inline(always)]
			fn write(value: Self::Write<'_>, mut buf: impl SerBuf) -> Result<()> {
				buf.write_all(&value.to_le_bytes()).err_into()
			}
		}

		impl LayoutRead for $t {
            type Read<'a> = Self;

			#[inline(always)]
			fn read(buf: &mut &[u8]) -> Result<Self> {
				if buf.len() < size_of::<Self>() {
					return Err(InvalidFormat(type_name::<Self>()).into());
				}
				let (read, rest) = buf.split_at(size_of::<Self>());
				let result = Self::from_le_bytes(unsafe { read.try_into().unwrap_unchecked() });
				*buf = rest;
				Ok(result)
			}
		}
	)*};
}

impl_nums!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

impl LayoutWrite for String {
	type Write<'a> = &'a str;

	#[inline(always)]
	fn size(value: Self::Write<'_>) -> Result<usize> {
		Ok(value.len() + size_of::<u32>())
	}

	#[inline(always)]
	fn write(value: Self::Write<'_>, mut buf: impl SerBuf) -> Result<()> {
		u32::write(value.len() as _, &mut buf)?;
		buf.write_all(value.as_bytes()).err_into()
	}
}

impl LayoutRead for String {
	type Read<'a> = &'a str;

	#[inline(always)]
	fn read<'a>(buf: &mut &'a [u8]) -> Result<Self::Read<'a>> {
		let len = u32::read(buf)? as _;
		if len > buf.len() {
			return Err(InvalidFormat(type_name::<Self>()).into());
		}
		let (read, rest) = buf.split_at(len);
		let result = std::str::from_utf8(read)?;
		*buf = rest;
		Ok(result)
	}
}

impl LayoutWrite for Vec<u8> {
	type Write<'a> = &'a [u8];

	#[inline(always)]
	fn size(value: Self::Write<'_>) -> Result<usize> {
		Ok(value.len() + size_of::<u32>())
	}

	#[inline(always)]
	fn write(value: Self::Write<'_>, mut buf: impl SerBuf) -> Result<()> {
		u32::write(value.len() as _, &mut buf)?;
		buf.write_all(value).err_into()
	}
}

impl LayoutRead for Vec<u8> {
	type Read<'a> = &'a [u8];

	#[inline(always)]
	fn read<'a>(buf: &mut &'a [u8]) -> Result<Self::Read<'a>> {
		let len = u32::read(buf)? as _;
		if len > buf.len() {
			return Err(InvalidFormat(type_name::<Self>()).into());
		}
		let (read, rest) = buf.split_at(len);
		*buf = rest;
		Ok(read)
	}
}

impl<T> LayoutWrite for Option<T>
where
	T: LayoutWrite,
{
	type Write<'a> = Option<T::Write<'a>>
	where
		Self:'a;

	#[inline(always)]
	fn size(value: Self::Write<'_>) -> Result<usize> {
		Ok(size_of::<u8>()
			+ if let Some(value) = value {
				T::size(value)?
			} else {
				0
			})
	}

	#[inline(always)]
	fn write(value: Self::Write<'_>, mut buf: impl SerBuf) -> Result<()> {
		if let Some(value) = value {
			u8::write(1, &mut buf)?;
			T::write(value, buf)
		} else {
			u8::write(0, buf)
		}
	}
}

impl<T> LayoutRead for Option<T>
where
	T: LayoutRead,
{
	type Read<'a> = Option<T::Read<'a>>;

	#[inline(always)]
	fn read<'a>(buf: &mut &'a [u8]) -> Result<Self::Read<'a>> {
		Ok(if u8::read(buf)? == 1 {
			Some(T::read(buf)?)
		} else {
			None
		})
	}
}

pub struct UseFromToBytes<T>(T);

impl<T> LayoutWrite for UseFromToBytes<T>
where
	T: ToBytes,
{
	type Write<'a> = &'a T
	where
		Self: 'a;

	#[inline(always)]
	fn size(value: Self::Write<'_>) -> Result<usize> {
		value.bytes_len().map(|x| x + size_of::<u32>()).err_into()
	}

	#[inline(always)]
	fn write(value: Self::Write<'_>, mut buf: impl SerBuf) -> Result<()> {
		u32::write(value.bytes_len()? as _, &mut buf)?;
		value.write_to(buf).err_into()
	}
}

impl<T> LayoutRead for UseFromToBytes<T>
where
	T: for<'a> FromBytes<'a>,
{
	type Read<'a> = T;

	#[inline(always)]
	fn read<'a>(buf: &mut &'a [u8]) -> Result<Self::Read<'a>> {
		let len = u32::read(buf)? as _;
		if len > buf.len() {
			return Err(InvalidFormat(type_name::<T>()).into());
		}
		let (read, rest) = buf.split_at(len);
		let result = T::from_bytes(read)?;
		*buf = rest;
		Ok(result)
	}
}

pub struct WithSize<T>(T);

impl<T> LayoutWrite for WithSize<T>
where
	T: LayoutWrite,
{
	type Write<'a> = T::Write<'a>
	where
		Self: 'a;

	#[inline(always)]
	fn size(value: Self::Write<'_>) -> Result<usize> {
		let size = T::size(value)?;
		Ok(u32::size(size as _)? + size)
	}

	#[inline(always)]
	fn write(value: Self::Write<'_>, mut buf: impl SerBuf) -> Result<()> {
		let size = T::size(value)?;
		u32::write(size as _, &mut buf)?;
		T::write(value, buf)
	}
}

impl<T> LayoutRead for WithSize<T>
where
	T: LayoutRead,
{
	type Read<'a> = T::Read<'a>;

	#[inline(always)]
	fn read<'a>(buf: &mut &'a [u8]) -> Result<Self::Read<'a>> {
		let size = u32::read(buf)? as usize;
		if size > buf.len() {
			return Err(InvalidFormat(type_name::<T>()).into());
		}
		T::read(buf)
	}
}

#[impl_for_tuples(64)]
impl LayoutWrite for Item {
	type Write<'a> = for_tuples!((#(Item::Write::<'a>),*))
	where
		Self: 'a;

	#[inline(always)]
	fn size(value: Self::Write<'_>) -> Result<usize> {
		let result = 0;
		for_tuples!(#(let result = result + Item::size(value.Item)?;)*);
		Ok(result)
	}

	#[inline(always)]
	fn write(value: Self::Write<'_>, mut buf: impl SerBuf) -> Result<()> {
		for_tuples!(#(
            Item::write(value.Item, &mut buf);
        )*);
		Ok(())
	}
}

#[impl_for_tuples(64)]
impl LayoutRead for Item {
	type Read<'a> = for_tuples!((#(Item::Read::<'a>),*));

	#[inline(always)]
	fn read<'a>(buf: &mut &'a [u8]) -> Result<Self::Read<'a>> {
		Ok((for_tuples!(#(Item::read(buf)?),*)))
	}
}
