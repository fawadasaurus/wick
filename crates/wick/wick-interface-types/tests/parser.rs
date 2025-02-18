#![cfg(feature = "parser")]

use anyhow::Result;
use wick_interface_types::{
  component,
  fields,
  operation,
  parse,
  ComponentSignature,
  Field,
  OperationSignature,
  TypeSignature,
};

#[test_log::test]
fn test_parser() -> Result<()> {
  assert_eq!(parse("bool")?, TypeSignature::Bool);
  assert_eq!(
    parse("bool[]")?,
    TypeSignature::List {
      ty: Box::new(TypeSignature::Bool),
    }
  );
  let fields = vec![Field::new("myBool", TypeSignature::Bool)];
  assert_eq!(parse("{ myBool : bool }")?, TypeSignature::AnonymousStruct(fields));

  let custom = TypeSignature::Custom("some_struct".to_owned());
  assert_eq!(parse("some_struct")?, custom);

  Ok(())
}

#[test_log::test]
fn test_typemap() -> Result<()> {
  let map = fields! {
    "myBool" => "bool",
  };
  let fields = vec![Field::new("myBool", TypeSignature::Bool)];
  assert_eq!(map, fields);

  Ok(())
}

#[test_log::test]
fn test_op_macro() -> Result<()> {
  let actual = operation! {
    "test-component" => {
      inputs: {"input"=>"string",},
      outputs: {"output"=>"string",},
    }
  };
  let expected = OperationSignature {
    name: "test-component".to_owned(),
    inputs: fields! {"input"=>"string"},
    outputs: fields! {"output"=>"string"},
  };
  assert_eq!(actual, expected);
  let actual = operation! {"math::subtract" => {
    inputs: { "left" => "u64", "right" => "u64",},
    outputs: { "output" => "u64" ,},
  }};
  let expected = OperationSignature {
    name: "math::subtract".to_owned(),
    inputs: fields! {"left"=>"u64","right"=>"u64"},
    outputs: fields! {"output"=>"u64"},
  };
  assert_eq!(actual, expected);

  Ok(())
}

#[test_log::test]
fn test_component_macro() -> Result<()> {
  let mut opmap = Vec::default();
  opmap.push(operation! {
    "test-component" => {
      inputs: {"input"=>"string"},
      outputs: {"output"=>"string"},
    }
  });

  let expected = ComponentSignature {
    name: Some("test-native-collection".to_owned()),
    operations: opmap,
    ..Default::default()
  };
  let actual = component! {
      name: "test-native-collection",
      version: "0.1.0",
      operations: {
        "test-component" => {
          inputs: {"input" => "string"},
          outputs: {"output" => "string"},
        }
      }
  };
  assert_eq!(actual, expected);
  Ok(())
}
