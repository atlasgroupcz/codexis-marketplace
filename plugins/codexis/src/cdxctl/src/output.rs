use serde_json::Value;

#[derive(Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Json,
    Table,
}

impl OutputFormat {
    pub fn from_flag(table: bool) -> Self {
        if table {
            OutputFormat::Table
        } else {
            OutputFormat::Json
        }
    }
}

/// Print a JSON value according to the chosen format.
pub fn print_output(value: &Value, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(value).unwrap_or_default()
            );
        }
        OutputFormat::Table => {
            print_table(value);
        }
    }
}

fn print_table(value: &Value) {
    match value {
        Value::Array(arr) => {
            if arr.is_empty() {
                println!("(no results)");
                return;
            }
            // Collect column names from the first object
            let columns = collect_columns(&arr[0]);
            if columns.is_empty() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(value).unwrap_or_default()
                );
                return;
            }
            // Calculate column widths
            let mut widths: Vec<usize> = columns.iter().map(|c| c.len()).collect();
            let rows: Vec<Vec<String>> = arr
                .iter()
                .map(|row| {
                    columns
                        .iter()
                        .enumerate()
                        .map(|(i, col)| {
                            let val = format_cell(row.get(col));
                            if val.len() > widths[i] {
                                widths[i] = val.len();
                            }
                            val
                        })
                        .collect()
                })
                .collect();
            // Cap widths at 50 chars
            for w in widths.iter_mut() {
                if *w > 50 {
                    *w = 50;
                }
            }
            // Print header
            let header: Vec<String> = columns
                .iter()
                .enumerate()
                .map(|(i, c)| format!("{:width$}", c.to_uppercase(), width = widths[i]))
                .collect();
            println!("{}", header.join("  "));
            let separator: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
            println!("{}", separator.join("  "));
            // Print rows
            for row in &rows {
                let formatted: Vec<String> = row
                    .iter()
                    .enumerate()
                    .map(|(i, val)| {
                        let truncated = if val.len() > widths[i] {
                            format!("{}…", &val[..widths[i] - 1])
                        } else {
                            val.clone()
                        };
                        format!("{:width$}", truncated, width = widths[i])
                    })
                    .collect();
                println!("{}", formatted.join("  "));
            }
        }
        Value::Object(_) => {
            // Single object: print key-value pairs
            for (key, val) in value.as_object().unwrap() {
                println!("{}: {}", key, format_cell(Some(val)));
            }
        }
        _ => {
            println!("{value}");
        }
    }
}

fn collect_columns(obj: &Value) -> Vec<String> {
    match obj.as_object() {
        Some(map) => map
            .keys()
            .filter(|k| {
                // Skip nested objects/arrays for table display
                match map.get(*k) {
                    Some(Value::Object(_)) | Some(Value::Array(_)) => false,
                    _ => true,
                }
            })
            .cloned()
            .collect(),
        None => vec![],
    }
}

fn format_cell(val: Option<&Value>) -> String {
    match val {
        None | Some(Value::Null) => "-".to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Array(arr)) => {
            let items: Vec<String> = arr.iter().map(|v| format_cell(Some(v))).collect();
            items.join(", ")
        }
        Some(Value::Object(_)) => "(object)".to_string(),
    }
}
