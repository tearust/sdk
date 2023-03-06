use actorx_example_codec::error::ExampleCodec;
use tea_sdk::{actorx::host::error::Host, define_scope};

define_scope! {
	ExampleHost: pub ExampleCodec, Host {}
}
