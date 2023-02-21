pub use tea_codec_macros::Priced;

#[rustc_specialization_trait]
pub trait Priced {
	fn price(&self) -> Option<u64>;
}

pub trait PricedOrDefault {
	fn price(&self) -> Option<u64>;
}

impl<T> PricedOrDefault for T {
	default fn price(&self) -> Option<u64> {
		None
	}
}

impl<T> PricedOrDefault for T
where
	T: Priced,
{
	fn price(&self) -> Option<u64> {
		Priced::price(self)
	}
}
