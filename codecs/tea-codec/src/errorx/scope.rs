use std::{any::TypeId, borrow::Cow};

use super::{Error, Global, SmallVec};

pub trait Scope: Send + Sync + 'static {
	type Parent: Scope;
	type Descriptor<T>: Descriptor<T>;
	const NAME: &'static str;
	const FULLNAME: &'static str;
}

pub trait Descriptor<T> {
	fn name(v: &T) -> Option<Cow<str>>;
	fn summary(v: &T) -> Option<Cow<str>>;
	fn detail(v: &T) -> Option<Cow<str>>;
	fn inner(v: &T) -> Option<SmallVec<[&Error; 1]>>;
	fn type_id(v: &T) -> Option<TypeId>;
}

#[marker]
pub trait DescriptableMark<T>
where
	T: ?Sized,
{
}

pub(crate) trait Descriptee: Send + Sync {
	fn name(&self) -> Option<Cow<str>>;
	fn summary(&self) -> Option<Cow<str>>;
	fn detail(&self) -> Option<Cow<str>>;
	fn inner(&self) -> Option<SmallVec<[&Error; 1]>>;
	fn type_id(&self) -> Option<TypeId>;
}

#[repr(transparent)]
pub(crate) struct Dispatcher<T> {
	pub(crate) data: T,
}

impl<T> Dispatcher<T> {
	pub fn new(data: T) -> Self {
		Self { data }
	}
}

impl<T> Descriptee for Dispatcher<T>
where
	T: Send + Sync,
{
	default fn name(&self) -> Option<Cow<str>> {
		<<Global as Scope>::Descriptor<T> as Descriptor<T>>::name(&self.data)
	}

	default fn summary(&self) -> Option<Cow<str>> {
		<<Global as Scope>::Descriptor<T> as Descriptor<T>>::summary(&self.data)
	}

	default fn detail(&self) -> Option<Cow<str>> {
		<<Global as Scope>::Descriptor<T> as Descriptor<T>>::detail(&self.data)
	}

	default fn inner(&self) -> Option<SmallVec<[&Error; 1]>> {
		<<Global as Scope>::Descriptor<T> as Descriptor<T>>::inner(&self.data)
	}

	default fn type_id(&self) -> Option<TypeId> {
		<<Global as Scope>::Descriptor<T> as Descriptor<T>>::type_id(&self.data)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::errorx::IntoError;

	#[test]
	fn specialize_descriptor_works() {
		#[derive(Debug, thiserror::Error)]
		#[error("has inner")]
		struct HasInner(Error);

		pub type Result<T, E = Error> = std::result::Result<T, E>;
		pub enum Test2 {
			HasInner,
		}

		impl From<Test2> for Cow<'static, str> {
			fn from(s: Test2) -> Self {
				s.name_const().into()
			}
		}

		impl<T> Descriptor<T> for Test2 {
			default fn name(v: &T) -> Option<Cow<str>> {
				if let Some(r) = <Global as Descriptor<T>>::name(v) {
					return Some(r);
				}
				None
			}

			default fn inner(v: &T) -> Option<SmallVec<[&Error; 1]>> {
				if let Some(r) = <Global as Descriptor<T>>::inner(v) {
					return Some(r);
				}
				None
			}
			default fn type_id(v: &T) -> Option<TypeId> {
				if let Some(r) = <Global as Descriptor<T>>::type_id(v) {
					return Some(r);
				}
				None
			}
			default fn summary(v: &T) -> Option<Cow<str>> {
				if let Some(r) = <Global as Descriptor<T>>::summary(v) {
					return Some(r);
				}
				Some(
					{
						let res = format!("{0}", ::tea_sdk::ImplDefault(v));
						res
					}
					.into(),
				)
			}
			default fn detail(v: &T) -> Option<Cow<str>> {
				if let Some(r) =
					<::tea_sdk::errorx::Global as ::tea_sdk::errorx::Descriptor<T>>::detail(v)
				{
					return Some(r);
				}
				Some(
					{
						let res = format!("{0:?}", ::tea_sdk::ImplDefault(v));
						res
					}
					.into(),
				)
			}
		}

		impl Descriptor<HasInner> for Test2 {
			fn name<'a>(_: &'a HasInner) -> Option<Cow<'a, str>> {
				Some(Test2::HasInner.into())
			}
			fn summary<'a>(x: &'a HasInner) -> Option<Cow<'a, str>> {
				Some(
					{
						let res = format!("{0:?}", x);
						res
					}
					.into(),
				)
			}
			fn detail<'a>(x: &'a HasInner) -> Option<Cow<'a, str>> {
				Some(
					{
						let res = format!("{0:?}", x);
						res
					}
					.into(),
				)
			}
			fn inner<'a>(x: &'a HasInner) -> Option<SmallVec<[&'a Error; 1]>> {
				Some({
					let mut inner = SmallVec::<[&Error; 1]>::new();
					inner.reserve_exact(1usize);
					inner.push((&x.0).into());
					inner
				})
			}
			fn type_id<'a>(_: &'a HasInner) -> Option<TypeId> {
				Some(TypeId::of::<HasInner>())
			}
		}

		impl Scope for Test2 {
			type Parent = Global;

			type Descriptor<T> = Self;

			const NAME: &'static str = "Test2";

			const FULLNAME: &'static str = {
				const N1: &[u8] = <Global as Scope>::NAME.as_bytes();
				const N2: &[u8] = <Test2 as Scope>::NAME.as_bytes();
				if let b"Global" = N1 {
					<Test2 as Scope>::NAME
				} else {
					const LEN: usize = N1.len() + N2.len() + 1;
					const fn combine() -> [u8; LEN] {
						let mut result = [0u8; LEN];
						let mut i = 0;
						while i < N1.len() {
							result[i] = N1[i];
							i += 1;
						}
						result[i] = b'.';
						i = 0;
						while i < N2.len() {
							result[N1.len() + 1 + i] = N2[i];
							i += 1;
						}
						result
					}
					unsafe { std::str::from_utf8_unchecked(&combine()) }
				}
			};
		}

		impl Test2 {
			pub const fn name_const(&self) -> &'static str {
				const HAS_INNER: &'static str = {
					const N1: &[u8] = <Test2 as ::tea_sdk::errorx::Scope>::FULLNAME.as_bytes();
					const N2: &[u8] = "HasInner".as_bytes();
					if let b"Global" = N1 {
						"HasInner"
					} else {
						const LEN: usize = N1.len() + N2.len() + 1;
						const fn combine() -> [u8; LEN] {
							let mut result = [0u8; LEN];
							let mut i = 0;
							while i < N1.len() {
								result[i] = N1[i];
								i += 1;
							}
							result[i] = b'.';
							i = 0;
							while i < N2.len() {
								result[N1.len() + 1 + i] = N2[i];
								i += 1;
							}
							result
						}
						unsafe { ::std::str::from_utf8_unchecked(&combine()) }
					}
				};
				match self {
					Test2::HasInner => HAS_INNER,
				}
			}
		}

		fn foo() -> Result<()> {
			baz()?;
			Ok(())
		}

		fn baz() -> Result<()> {
			Err(HasInner(Error::new("inner error")).into_error())
		}

		// additonal specialization
		impl Descriptee for Dispatcher<HasInner> {
			fn name(&self) -> Option<Cow<str>> {
				<Test2 as Descriptor<HasInner>>::name(&self.data)
			}

			fn summary(&self) -> Option<Cow<str>> {
				todo!()
			}

			fn detail(&self) -> Option<Cow<str>> {
				todo!()
			}

			fn inner(&self) -> Option<SmallVec<[&Error; 1]>> {
				todo!()
			}

			fn type_id(&self) -> Option<TypeId> {
				todo!()
			}
		}

		let ex = foo().unwrap_err();
		assert_eq!(ex.name(), "Test2.HasInner");
		assert_eq!(ex.name(), Test2::HasInner.name_const());
	}
}
