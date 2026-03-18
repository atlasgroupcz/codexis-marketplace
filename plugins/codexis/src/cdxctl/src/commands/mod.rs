pub mod agent;
pub mod automation;
pub mod marketplace;
pub mod notification;
pub mod plugin;
pub mod skill;
pub mod tabular;

use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;

/// Resolve an ID that may be either a raw UUID or a base64-encoded Node ID.
/// If it looks like a UUID (contains only hex digits and dashes), encode it as a Node ID.
/// Otherwise, assume it's already a Node ID.
pub fn resolve_node_id(id: &str, type_prefix: &str) -> String {
    if is_node_id(id, type_prefix) {
        id.to_string()
    } else if is_uuid(id) {
        STANDARD_NO_PAD.encode(format!("{type_prefix}:{id}"))
    } else {
        STANDARD_NO_PAD.encode(format!("{type_prefix}:{id}"))
    }
}

fn is_uuid(s: &str) -> bool {
    s.len() == 36
        && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
        && s.chars().filter(|c| *c == '-').count() == 4
}

fn is_node_id(s: &str, type_prefix: &str) -> bool {
    STANDARD_NO_PAD
        .decode(s)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .is_some_and(|decoded| decoded.starts_with(&format!("{type_prefix}:")))
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

    #[test]
    fn test_resolve_named_id_to_node_id() {
        let result = resolve_node_id("test", "Skill");
        let decoded = String::from_utf8(STANDARD_NO_PAD.decode(&result).unwrap()).unwrap();
        assert_eq!(decoded, "Skill:test");
    }

    #[test]
    fn test_detects_named_skill_node_id() {
        let node_id = STANDARD_NO_PAD.encode("Skill:test");
        assert_eq!(resolve_node_id(&node_id, "Skill"), node_id);
    }
}
