use crate::cli::OutputFormat;
use anyhow::{Context, Result};
use comfy_table::Table;
use csv::WriterBuilder;
use serde_json::Value;
use std::collections::HashMap;

pub fn serialize(results: &[HashMap<String, Value>], format: OutputFormat) -> Result<String> {
    if results.is_empty() {
        return Ok(String::new());
    }

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(results).context("Failed to serialize to JSON")
        }
        OutputFormat::Csv => {
            let mut wtr = WriterBuilder::new().from_writer(vec![]);

            // Get headers from the first record
            let headers: Vec<&String> = results[0].keys().collect();
            wtr.write_record(&headers)?;

            for record in results {
                let mut row = Vec::new();
                for header in &headers {
                    let val = record.get(*header).cloned().unwrap_or(Value::Null);
                    row.push(json_value_to_string(&val));
                }
                wtr.write_record(&row)?;
            }

            let data =
                String::from_utf8(wtr.into_inner()?).context("Failed to convert CSV to UTF-8")?;
            Ok(data)
        }
        OutputFormat::Table => {
            let mut table = Table::new();

            // Headers
            let headers: Vec<&String> = results[0].keys().collect();
            table.set_header(&headers);

            for record in results {
                let mut row = Vec::new();
                for header in &headers {
                    let val = record.get(*header).cloned().unwrap_or(Value::Null);
                    row.push(json_value_to_string(&val));
                }
                table.add_row(row);
            }

            Ok(table.to_string())
        }
    }
}

fn json_value_to_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            // Join list values with newline for better table/csv representation
            arr.iter()
                .map(|v| json_value_to_string(v))
                .collect::<Vec<String>>()
                .join(
                    "
",
                )
        }
        _ => val.to_string(),
    }
}
