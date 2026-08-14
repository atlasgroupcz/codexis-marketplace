use serde_json::Value;

/// Widest a column is allowed to get before its cells are truncated.
const MAX_COLUMN_WIDTH: usize = 50;

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
            println!("{}", render_table(value));
        }
    }
}

/// How wide a string is, in the only unit this module may use.
///
/// Characters, because `{:width$}` pads by character count. Not display columns:
/// a CJK ideograph is one character and two columns, so those draw crooked.
fn width_of(s: &str) -> usize {
    s.chars().count()
}

/// A cell padded to `width`, with the ellipsis counting as one of those
/// characters, so a truncated cell is exactly `width` wide.
///
/// Cut with `{:.*}`, which cuts by character. `&value[..n]` cuts by byte and
/// panics mid-character.
fn fit(value: &str, width: usize) -> String {
    if width_of(value) > width {
        format!("{:.*}…", width.saturating_sub(1), value)
    } else {
        format!("{value:width$}")
    }
}

fn render_table(value: &Value) -> String {
    match value {
        Value::Array(arr) => render_rows(arr),
        Value::Object(map) => map
            .iter()
            .map(|(key, val)| format!("{}: {}", key, format_cell(Some(val))))
            .collect::<Vec<_>>()
            .join("\n"),
        other => other.to_string(),
    }
}

fn render_rows(arr: &[Value]) -> String {
    if arr.is_empty() {
        return "(no results)".to_string();
    }
    // Column names come from the first object; a later row missing one of them
    // reads as "-" rather than shifting the table.
    let columns = collect_columns(&arr[0]);
    if columns.is_empty() {
        return serde_json::to_string_pretty(arr).unwrap_or_default();
    }

    // Measured after uppercasing, because that is the string being printed.
    let headers: Vec<String> = columns.iter().map(|c| c.to_uppercase()).collect();
    let rows: Vec<Vec<String>> = arr
        .iter()
        .map(|row| {
            columns
                .iter()
                .map(|col| format_cell(row.get(col)))
                .collect()
        })
        .collect();
    let widths: Vec<usize> = headers
        .iter()
        .enumerate()
        .map(|(i, header)| {
            rows.iter()
                .map(|row| width_of(&row[i]))
                .fold(width_of(header), usize::max)
                .min(MAX_COLUMN_WIDTH)
        })
        .collect();

    // Header, rule and body are the same operation, so they go through one pass.
    let rule: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
    [&headers, &rule]
        .into_iter()
        .chain(rows.iter())
        .map(|row| {
            row.iter()
                .zip(&widths)
                .map(|(cell, width)| fit(cell, *width))
                .collect::<Vec<_>>()
                .join("  ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect_columns(obj: &Value) -> Vec<String> {
    match obj.as_object() {
        Some(map) => map
            .keys()
            .filter(|k| {
                // Skip nested objects/arrays for table display
                !matches!(map.get(*k), Some(Value::Object(_)) | Some(Value::Array(_)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 52 characters in 56 bytes. Both halves matter and neither is typical of a
    /// Czech sentence this long, so the test asserts the shape before using it.
    const CZECH: &str = "Práce s tabulkami Excel a CSV, výpočty a analýzy dat";

    fn lines(value: &Value) -> Vec<String> {
        render_table(value).lines().map(|l| l.to_string()).collect()
    }

    #[test]
    fn truncates_czech_text_to_exactly_the_column_width() {
        // Reword the fixture and this silently stops covering anything: it needs
        // to be long enough to truncate, and the old byte cut has to land inside
        // a character rather than between two.
        assert!(width_of(CZECH) > MAX_COLUMN_WIDTH, "too short to truncate");
        assert!(
            !CZECH.is_char_boundary(MAX_COLUMN_WIDTH - 1),
            "the old cut would not have panicked here"
        );

        let rendered = lines(&json!([{ "description": CZECH }]));

        assert!(rendered[2].contains('…'), "{}", rendered[2]);
        assert_eq!(rendered[2].chars().count(), MAX_COLUMN_WIDTH);
    }

    #[test]
    fn a_column_is_as_wide_as_its_widest_value_in_characters() {
        // "žluťoučký" is 9 characters in 13 bytes, so byte measurement draws
        // every one of these lines 4 columns wider than it needs to be.
        let rendered = lines(&json!([{ "name": "žluťoučký" }, { "name": "a" }]));

        assert!(
            rendered.iter().all(|l| l.chars().count() == 9),
            "{rendered:#?}"
        );
    }

    #[test]
    fn a_column_missing_from_a_later_row_reads_as_a_dash() {
        let rendered = lines(&json!([{ "id": "1", "name": "a" }, { "id": "2" }]));

        assert!(rendered[3].contains('-'), "{}", rendered[3]);
    }

    #[test]
    fn an_empty_array_says_so() {
        assert_eq!(render_table(&json!([])), "(no results)");
    }

    #[test]
    fn a_single_object_prints_key_value_pairs() {
        let rendered = render_table(&json!({ "name": "žluťoučký", "seen": true }));

        assert_eq!(rendered, "name: žluťoučký\nseen: true");
    }
}
