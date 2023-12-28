#[cfg(any(feature = "host", feature = "worker"))]
use std::time::Duration;

#[cfg(any(feature = "host", feature = "worker"))]
use strum::{Display, FromRepr};
use tea_sdk::errorx::IntoError;
#[cfg(any(feature = "wasm", feature = "worker"))]
use tea_sdk::serde::error::InvalidFormat;
#[cfg(any(feature = "host", feature = "worker"))]
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::error::{Error, Result};

#[cfg(all(any(feature = "host", feature = "worker"), not(feature = "nitro")))]
pub const WORKER_UNIX_SOCKET_FD: i32 = 10;

#[cfg(any(feature = "host", feature = "worker"))]
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, Hash, FromRepr)]
#[repr(u8)]
pub enum OpCode {
	Call = 0,
	ReturnOk = 1,
	ReturnErr = 2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
	Call { ctx: Vec<u8>, req: Vec<u8> },
	ReturnOk { resp: Vec<u8> },
	ReturnErr { error: Error },
}

#[cfg(any(feature = "wasm", feature = "worker"))]
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct OperationAbi {
	pub flag: u8,
	pub data_0: u32,
	pub len_0: u32,
	pub data_1: u32,
	pub len_1: u32,
}

#[cfg(any(feature = "wasm", feature = "worker"))]
impl OperationAbi {
	#[cfg(feature = "wasm")]
	pub unsafe fn set_flag(&mut self, flag: u8) {
		self.flag = flag;
	}

	pub unsafe fn marshal(
		&mut self,
		op: Operation,
		mut handle_vec: impl FnMut(Vec<u8>, &mut u32, &mut u32) -> Result<(), Error>,
	) -> Result<()> {
		match op {
			Operation::Call { ctx, req } => {
				self.flag = 0;
				handle_vec(ctx, &mut self.data_0, &mut self.len_0)?;
				handle_vec(req, &mut self.data_1, &mut self.len_1)?;
			}
			Operation::ReturnOk { resp } => {
				self.flag = 1;
				handle_vec(resp, &mut self.data_0, &mut self.len_0)?;
			}
			Operation::ReturnErr { error } => {
				let error = bincode::serialize(&error)?;
				self.flag = 2;
				handle_vec(error, &mut self.data_0, &mut self.len_0)?;
			}
		}
		Ok(())
	}

	pub unsafe fn unmarshal(
		&self,
		mut handle_vec: impl FnMut(u32, u32) -> Result<Vec<u8>, Error>,
	) -> Result<Operation> {
		let vec_0 = handle_vec(self.data_0, self.len_0)?;
		match self.flag {
			0 => Ok(Operation::Call {
				ctx: vec_0,
				req: handle_vec(self.data_1, self.len_1)?,
			}),
			1 => Ok(Operation::ReturnOk { resp: vec_0 }),
			2 => Ok(Operation::ReturnErr {
				error: bincode::deserialize(&vec_0)?,
			}),
			_ => Err(InvalidFormat.into_error()),
		}
	}

	#[cfg(feature = "wasm")]
	#[allow(clippy::uninit_vec)]
	pub unsafe fn alloc_0(&mut self, len: usize) {
		let mut vec = Vec::<u8>::with_capacity(len);
		vec.set_len(len);
		self.data_0 = Box::into_raw(vec.into_boxed_slice()) as *mut u8 as _;
		self.len_0 = len as _;
	}

	#[cfg(feature = "wasm")]
	#[allow(clippy::uninit_vec)]
	pub unsafe fn alloc_1(&mut self, len: usize) {
		let mut vec = Vec::<u8>::with_capacity(len);
		vec.set_len(len);
		self.data_1 = Box::into_raw(vec.into_boxed_slice()) as *mut u8 as _;
		self.len_1 = len as _;
	}

	#[cfg(feature = "wasm")]
	pub unsafe fn dealloc(&mut self) {
		match self.flag {
			0 => drop(Box::from_raw(std::slice::from_raw_parts_mut(
				self.data_1 as *mut u8,
				self.len_1 as _,
			))),
			1 | 2 => (),
			flag => unreachable!("dealloc abi with flag {flag}"),
		}
		drop(Box::from_raw(std::slice::from_raw_parts_mut(
			self.data_0 as *mut u8,
			self.len_0 as _,
		)));
		*self = Default::default();
	}
}

#[cfg(any(feature = "wasm", feature = "worker"))]
impl Default for OperationAbi {
	fn default() -> Self {
		Self {
			flag: 3,
			data_0: 0,
			len_0: 0,
			data_1: 0,
			len_1: 0,
		}
	}
}

#[cfg(any(feature = "host", feature = "worker"))]
impl Operation {
	pub async fn read<R>(mut read: R, code: u8) -> Result<Result<(Self, u64, u64), u8>>
	where
		R: AsyncRead + Unpin,
	{
		let cid = read.read_u64_le().await?;
		let gas = read.read_u64_le().await?;
		let data_0 = read_var_bytes(&mut read).await?;
		Ok(match OpCode::from_repr(code) {
			Some(OpCode::Call) => {
				let ctx = data_0;
				let req = read_var_bytes(read).await?;
				Ok((Self::Call { ctx, req }, cid, gas))
			}
			Some(OpCode::ReturnOk) => Ok((Self::ReturnOk { resp: data_0 }, cid, gas)),
			Some(OpCode::ReturnErr) => Ok((
				Self::ReturnErr {
					error: bincode::deserialize(&data_0)?,
				},
				cid,
				gas,
			)),
			_ => Err(code),
		})
	}

	pub async fn read_code<R>(mut read: R) -> Result<Option<u8>>
	where
		R: AsyncRead + Unpin,
	{
		Ok(tokio::select! {
			r = read.read_u8() => Some(r?),
			_ = tokio::time::sleep(Duration::from_secs(5)) => None,
		})
	}

	pub async fn write<W>(&self, mut write: W, cid: u64, gas: u64) -> Result<()>
	where
		W: AsyncWrite + Unpin,
	{
		match self {
			Operation::Call { ctx, req } => {
				write.write_u8(OpCode::Call as _).await?;
				write.write_u64_le(cid).await?;
				write.write_u64_le(gas).await?;
				write_var_bytes(&mut write, ctx).await?;
				write_var_bytes(write, req).await?;
			}
			Operation::ReturnOk { resp } => {
				write.write_u8(OpCode::ReturnOk as _).await?;
				write.write_u64_le(cid).await?;
				write.write_u64_le(gas).await?;
				write_var_bytes(write, resp).await?;
			}
			Operation::ReturnErr { error } => {
				write.write_u8(OpCode::ReturnErr as _).await?;
				write.write_u64_le(cid).await?;
				write.write_u64_le(gas).await?;
				write_var_bytes(write, &bincode::serialize(error)?).await?;
			}
		}
		Ok(())
	}
}

#[cfg(any(feature = "host", feature = "worker"))]
#[allow(clippy::uninit_vec)]
pub async fn read_var_bytes<R>(mut read: R) -> Result<Vec<u8>>
where
	R: AsyncRead + Unpin,
{
	let len = read.read_u64_le().await? as _;
	let mut buf = Vec::with_capacity(len);
	unsafe {
		buf.set_len(len);
	}
	read.read_exact(&mut buf).await?;
	Ok(buf)
}

#[cfg(any(feature = "host", feature = "worker"))]
pub async fn write_var_bytes<W>(mut write: W, bytes: &[u8]) -> Result<()>
where
	W: AsyncWrite + Unpin,
{
	write.write_u64_le(bytes.len() as _).await?;
	write.write_all(bytes).await?;
	Ok(())
}
