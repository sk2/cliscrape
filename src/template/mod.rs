pub mod loader;
pub mod modern;
pub mod convert;

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "template/textfsm.pest"]
pub struct TextFsmParser;
