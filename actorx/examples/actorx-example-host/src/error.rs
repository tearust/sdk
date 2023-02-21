use actorx_example_codec::error::ExampleCodec;
use tea_actorx_host::error::Host;
use tea_codec::define_scope;

define_scope! {
	ExampleHost: pub ExampleCodec, Host {}
}
