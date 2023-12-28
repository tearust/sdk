// use crate::{
// 	deserialize,
// 	errorx::{define_scope, Global},
// 	serialize,
// };

// define_scope! {
// 	Test: pub Test3, Test2 {
// 		Foo;
// 		NotImplemented => NotImplemented;
// 	}

// 	Test2 {
// 		I32;
// 		i32 as v => @Test2::I32, @Debug, v.to_string();
// 		bool => Bool, @Serde;
// 		HasInner as x => HasInner, @Debug, @Debug, [&x.0];
// 	}

// 	Test3 {
// 	}
// }

// mod x {
// 	use tea_codec_macros::define_scope;

// 	define_scope! {}
// }

// #[derive(Debug)]
// struct HasInner(Error);

// #[derive(Debug)]
// struct NotImplemented(String, usize);

// #[test]
// fn test() {
// 	assert_eq!(
// 		Error::<Test>::from(NotImplemented("Yes".into(), 123)).detail(),
// 		Some("NotImplemented(\"Yes\", 123)".into())
// 	);
// 	assert_eq!(Test::Foo.name_const(), "Test3.Test.Foo");
// 	let ex = foo().unwrap_err();
// 	let s = serde_json::to_string(&ex).unwrap();
// 	let e: Error = serde_json::from_str(&s).unwrap();
// 	let s = serialize(&e).unwrap();
// 	let e: Error = deserialize(s).unwrap();
// 	assert_eq!(e.name(), Test2::HasInner);
// 	assert_eq!(e.inner()[0].name(), "Test2.I32");
// 	assert_eq!(e.inner()[0].detail(), Some("123".into()));
// 	let e = ex.back_cast::<HasInner>().unwrap();
// 	assert_eq!(e.0.back_cast_ref::<i32>().unwrap(), &123);
// 	assert_eq!(e.0.back_cast::<i32>().unwrap(), 123);
// 	let sum = bar().unwrap_err() + bar().unwrap_err() + bar().unwrap_err();
// 	assert_eq!(sum.inner().len(), 3);
// 	let sum = serde_json::from_str::<Error>(
// 		serde_json::to_string(&(bar().unwrap_err() + bar().unwrap_err()))
// 			.unwrap()
// 			.as_str(),
// 	)
// 	.unwrap()
// 		+ bar().unwrap_err();
// 	assert_eq!(sum.inner().len(), 3);
// 	assert_eq!(foobar().unwrap_err().name(), Global::Unknown);
// }

// fn foo() -> Result<()> {
// 	baz()?;
// 	Ok(())
// }

// fn bar() -> Result<(), Error> {
// 	Err(123i32.into())
// }

// fn baz() -> Result<()> {
// 	Err(HasInner(bar().unwrap_err().into()).into())
// }

// fn foobar() -> Result<(), Error> {
// 	Err("aaa".into())
// }
