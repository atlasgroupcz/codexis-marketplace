pub mod agent;
pub mod appenv;
pub mod automation;
pub mod bug;
pub mod marketplace;
pub mod notification;
pub mod notification_email;
pub mod package;
pub mod plugin;
pub mod schedule;
pub mod skill;
pub mod tabular;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::Value;

/// Find an entity's `id` in a list by matching `needle` against its `id` or any
/// of `match_fields` (e.g. `fullName`, `name`). Used to turn a human-supplied
/// agent/skill name into the Node `ID` the daemon's mutation inputs expect.
pub fn find_node_id(items: &[Value], needle: &str, match_fields: &[&str]) -> Option<Value> {
    items
        .iter()
        .find(|item| {
            item.get("id").and_then(Value::as_str) == Some(needle)
                || match_fields
                    .iter()
                    .any(|field| item.get(*field).and_then(Value::as_str) == Some(needle))
        })
        .and_then(|item| item.get("id").cloned())
}

/// Flatten a daemon entity's nested `plugin { name marketplace { name } }`
/// selection into top-level `plugin` / `marketplace` name strings, so they show
/// up as columns in `--table` output and as flat keys in JSON.
pub fn flatten_plugin_marketplace(value: &mut Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    let plugin = object.remove("plugin").unwrap_or(Value::Null);
    let plugin_name = plugin.get("name").cloned().unwrap_or(Value::Null);
    let marketplace_name = plugin
        .get("marketplace")
        .and_then(|marketplace| marketplace.get("name"))
        .cloned()
        .unwrap_or(Value::Null);
    object.insert("plugin".into(), plugin_name);
    object.insert("marketplace".into(), marketplace_name);
}

/// Encode a single-segment NodeId for the daemon's GraphQL `ID` scalar.
///
/// Wire format (matches `cdx-daemon` `NodeIds.encode`):
///     varint(len(typeName)) || typeName || varint(len(id)) || id
/// then URL-safe base64 without padding.
pub fn encode_node_id(type_name: &str, id: &str) -> String {
    let mut buf = Vec::with_capacity(type_name.len() + id.len() + 4);
    write_varint(&mut buf, type_name.len() as u64);
    buf.extend_from_slice(type_name.as_bytes());
    write_varint(&mut buf, id.len() as u64);
    buf.extend_from_slice(id.as_bytes());
    URL_SAFE_NO_PAD.encode(&buf)
}

/// Resolve an ID that may already be a daemon-encoded NodeId of `type_prefix`,
/// or a raw identifier (UUID, agent name, folder path, …) that should be wrapped
/// into a fresh root NodeId for `type_prefix`.
pub fn resolve_node_id(id: &str, type_prefix: &str) -> String {
    if root_type_name(id).as_deref() == Some(type_prefix) {
        id.to_string()
    } else {
        encode_node_id(type_prefix, id)
    }
}

fn write_varint(buf: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        buf.push(((value as u8) & 0x7F) | 0x80);
        value >>= 7;
    }
    buf.push(value as u8);
}

fn read_varint(bytes: &[u8], idx: &mut usize) -> Option<u64> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        if *idx >= bytes.len() || shift >= 64 {
            return None;
        }
        let byte = bytes[*idx];
        *idx += 1;
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Some(result);
        }
        shift += 7;
    }
}

/// Return the typeName of the root segment if `encoded` decodes as a NodeId.
fn root_type_name(encoded: &str) -> Option<String> {
    let bytes = URL_SAFE_NO_PAD.decode(encoded).ok()?;
    let mut idx = 0;
    let type_len = read_varint(&bytes, &mut idx)? as usize;
    if idx + type_len > bytes.len() {
        return None;
    }
    let type_bytes = bytes[idx..idx + type_len].to_vec();
    idx += type_len;
    let id_len = read_varint(&bytes, &mut idx)? as usize;
    if idx + id_len > bytes.len() {
        return None;
    }
    String::from_utf8(type_bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn find_node_id_matches_full_name_or_id() {
        let items = vec![
            json!({ "id": "Agent:enc-explorer", "fullName": "codexis:explorer", "name": "explorer" }),
            json!({ "id": "Agent:enc-writer", "fullName": "writer", "name": "writer" }),
        ];
        assert_eq!(
            find_node_id(&items, "codexis:explorer", &["fullName", "name"]),
            Some(json!("Agent:enc-explorer"))
        );
        assert_eq!(
            find_node_id(&items, "writer", &["fullName", "name"]),
            Some(json!("Agent:enc-writer"))
        );
        assert_eq!(
            find_node_id(&items, "Agent:enc-explorer", &["fullName", "name"]),
            Some(json!("Agent:enc-explorer"))
        );
        assert_eq!(find_node_id(&items, "missing", &["fullName", "name"]), None);
    }

    #[test]
    fn flatten_plugin_marketplace_lifts_names_to_top_level() {
        let mut value = json!({
            "id": "Agent:x",
            "plugin": { "id": "Plugin:p", "name": "codexis", "marketplace": { "id": "Marketplace:m", "name": "codexis-official" } }
        });
        flatten_plugin_marketplace(&mut value);
        assert_eq!(value["plugin"], json!("codexis"));
        assert_eq!(value["marketplace"], json!("codexis-official"));
    }

    #[test]
    fn flatten_plugin_marketplace_handles_local_entities() {
        // LOCAL agents/skills have no plugin → both columns null, no panic.
        let mut value = json!({ "id": "Agent:local", "plugin": Value::Null });
        flatten_plugin_marketplace(&mut value);
        assert_eq!(value["plugin"], Value::Null);
        assert_eq!(value["marketplace"], Value::Null);
    }

    #[test]
    fn encodes_skill_root_node_id() {
        // varint(5) "Skill" varint(4) "test" → BVNraWxsBHRlc3Q
        let encoded = encode_node_id("Skill", "test");
        assert_eq!(encoded, "BVNraWxsBHRlc3Q");
    }

    #[test]
    fn round_trips_short_identifiers() {
        for (ty, id) in [
            ("Automation", "550e8400-e29b-41d4-a716-446655440000"),
            ("Agent", "explorer"),
            ("Notification", "1700000000000"),
            ("TabularExtraction", "/home/codexis/projects/invoices"),
            ("Marketplace", "codexis-official"),
        ] {
            let encoded = encode_node_id(ty, id);
            assert_eq!(root_type_name(&encoded).as_deref(), Some(ty));
        }
    }

    #[test]
    fn resolves_raw_identifier_into_node_id() {
        let resolved = resolve_node_id("550e8400-e29b-41d4-a716-446655440000", "Automation");
        assert_eq!(root_type_name(&resolved).as_deref(), Some("Automation"));
    }

    #[test]
    fn passes_through_already_encoded_node_id() {
        let encoded = encode_node_id("Skill", "demo");
        assert_eq!(resolve_node_id(&encoded, "Skill"), encoded);
    }

    #[test]
    fn re_encodes_when_type_prefix_does_not_match() {
        let encoded_for_skill = encode_node_id("Skill", "demo");
        let resolved = resolve_node_id(&encoded_for_skill, "Agent");
        // The whole base64 string is now treated as a raw id under the Agent type.
        assert_eq!(root_type_name(&resolved).as_deref(), Some("Agent"));
    }

    #[test]
    fn varint_handles_lengths_above_127() {
        let mut buf = Vec::new();
        write_varint(&mut buf, 300);
        let mut idx = 0;
        assert_eq!(read_varint(&buf, &mut idx), Some(300));
        assert_eq!(idx, buf.len());
    }
}
