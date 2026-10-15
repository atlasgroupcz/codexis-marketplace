use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{ContentArrangement, Table};
use serde::Serialize;

use crate::cli::OutputFormat;

pub struct Output {
    format: OutputFormat,
}

impl Output {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Prints `value` as JSON, or calls `table` to print it for people.
    pub fn print<T: Serialize>(&self, value: &T, table: impl FnOnce(&T)) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => println!("{}", serde_json::to_string_pretty(value)?),
            OutputFormat::Table => table(value),
        }
        Ok(())
    }

    pub fn message(&self, text: &str) {
        if self.format == OutputFormat::Table {
            println!("{text}");
        }
    }
}

pub fn table<const N: usize>(
    header: [&str; N],
    rows: impl IntoIterator<Item = [String; N]>,
) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(header);
    for row in rows {
        table.add_row(row);
    }
    table
}

pub fn text(value: &Option<String>) -> String {
    value.clone().unwrap_or_else(|| "-".to_owned())
}
