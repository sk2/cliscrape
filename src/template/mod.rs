pub mod loader;
pub mod modern;

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "template/textfsm.pest"]
pub struct TextFsmParser;
