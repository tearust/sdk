use actorx_example_codec::error::ExampleCodec;
use tea_sdk::define_scope;

define_scope! {
	ExampleActor: ExampleCodec {
	}
}
