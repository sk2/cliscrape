pub mod convert;
pub mod library;
pub mod loader;
pub mod metadata;
pub mod modern;
pub mod resolver;

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "template/textfsm.pest"]
pub struct TextFsmParser;
