pub mod automation;
pub mod marketplace;
pub mod plugin;
pub mod tabular;

use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;

/// Resolve an ID that may be either a raw UUID or a base64-encoded Node ID.
/// If it looks like a UUID (contains only hex digits and dashes), encode it as a Node ID.
/// Otherwise, assume it's already a Node ID.
pub fn resolve_node_id(id: &str, type_prefix: &str) -> String {
    // If it looks like a UUID (hex + dashes, 36 chars), encode as Node ID
    if is_uuid(id) {
        STANDARD_NO_PAD.encode(format!("{type_prefix}:{id}"))
    } else {
        id.to_string()
    }
}

fn is_uuid(s: &str) -> bool {
    s.len() == 36
        && s.chars()
            .all(|c| c.is_ascii_hexdigit() || c == '-')
        && s.chars().filter(|c| *c == '-').count() == 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_detection() {
        assert!(is_uuid("550e8400-e29b-41d4-a716-446655440000"));
        assert!(!is_uuid("QXV0b21hdGlvbjoxMjM"));
        assert!(!is_uuid("short"));
    }

    #[test]
    fn test_resolve_uuid_to_node_id() {
        let result = resolve_node_id("550e8400-e29b-41d4-a716-446655440000", "Automation");
        // Should be base64 of "Automation:550e8400-e29b-41d4-a716-446655440000"
        let decoded = String::from_utf8(STANDARD_NO_PAD.decode(&result).unwrap()).unwrap();
        assert_eq!(decoded, "Automation:550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_resolve_already_node_id() {
        let node_id = "QXV0b21hdGlvbjoxMjM";
        assert_eq!(resolve_node_id(node_id, "Automation"), node_id);
    }
}
