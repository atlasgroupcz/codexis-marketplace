# Tabular Data Query Parameters

Query CSV/XLSX resources directly via the Tabular API. Works for files up to 100 MB (CSV) or 12.5 MB (XLSX).

## resource_id
Resource ID. Required. Get this from `list_dataset_resources` or `get_resource_info`.

## page
Page number. Optional, default: 1.

## page_size
Results per page. Optional, default: 20, range: 1-200.

## Filtering

### filter_column
Column name to filter on. Optional. Must be used together with `filter_value`.

### filter_value
Value to match. Optional. Must be used together with `filter_column`.

### filter_operator
Comparison operator. Optional, default: `exact`.

Available operators:
- `exact` — Exact match
- `contains` — Substring match (case-insensitive)
- `less` — Less than or equal
- `greater` — Greater than or equal
- `strictly_less` — Strictly less than
- `strictly_greater` — Strictly greater than

## Sorting

### sort_column
Column name to sort by. Optional.

### sort_direction
Sort direction. Optional, default: `asc`.
- `asc` — Ascending
- `desc` — Descending

## Tips

- Use `get_resource_info` first to check Tabular API availability
- Column names are returned in query results — use them for filtering and sorting
- For datasets with >1000 rows, use filtering to narrow results
- Values longer than 100 characters are truncated in results
