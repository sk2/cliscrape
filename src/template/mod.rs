pub mod loader;

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "template/textfsm.pest"]
pub struct TextFsmParser;
