use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "template/textfsm.pest"]
pub struct TextFsmParser;

fn main() {
    let input = r#"Start
  ^rule1 -> Continue.Record NextState
  ^rule2 -> Clear
  ^rule3 -> NextState
"#;
    match TextFsmParser::parse(Rule::file, input) {
        Ok(pairs) => {
            for pair in pairs {
                println!("{:?}", pair);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
