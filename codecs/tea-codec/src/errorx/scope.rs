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
