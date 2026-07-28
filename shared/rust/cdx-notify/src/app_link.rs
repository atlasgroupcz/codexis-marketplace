//! Resolve the in-app notification deep link for a plugin's own app.
//!
//! The notification bell navigates relative links through the frontend router,
//! and the only route serving a plugin app is `/apps/$applicationId`. That
//! `applicationId` is the daemon's opaque node id - not the plugin's name - so a
//! hand-written `/<slug>` link resolves to no route and the user lands on a 404.
//!
//! The id embeds the marketplace install uuid, which the plugin has no way to
//! know: it is absent from the environment and from `~/.cdx/plugins` state. So
//! it is looked up at send time from the daemon's `applications` query, which is
//! scoped to the signed-in user - the same principal the plugin's daemon token
//! already authenticates as for `me` and `createNotification`.
//!
//! The id itself is never parsed, only carried: the query also returns
//! `contentUrl`, which the daemon builds as
//! `/rest/plugin-components/<pluginNodeId>/<appId>/`, so the component id is
//! readable as its last path segment. Matching on that avoids decoding the
//! daemon's internal node-id encoding, which this repo has no contract with.
//!
//! Best-effort throughout: any missing config, transport error or uninstalled
//! app yields `None`, and the caller then sends a notification with no link.
//! An unclickable bell item beats one that navigates into a 404.

use crate::notify::graphql;

// ── application selection ─────────────────────────────────────────────────────

/// The component id a `contentUrl` points at - its last path segment.
///
/// The daemon percent-encodes that segment, but component ids are kebab-case
/// ASCII (repo convention), for which the encoding is the identity, so the
/// segment is compared as-is rather than pulling in a percent-decoder.
fn component_id_of(content_url: &str) -> Option<&str> {
    let trimmed = content_url.trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    trimmed.rsplit('/').next()
}

/// Pick the application id belonging to `plugin_name`'s `component_id` from an
/// `applications` query result.
///
/// Both halves matter: a plugin may publish several components (codexis ships
/// `sledovana-judikatura` and `sledovane-dokumenty`), and two marketplaces can
/// carry same-named components.
fn pick_application(
    applications: &serde_json::Value,
    plugin_name: &str,
    component_id: &str,
) -> Option<String> {
    applications.as_array()?.iter().find_map(|app| {
        let id = app.get("id")?.as_str()?;
        if app.get("plugin")?.get("name")?.as_str()? != plugin_name {
            return None;
        }
        let content_url = app.get("contentUrl")?.as_str()?;
        (component_id_of(content_url)? == component_id).then(|| id.to_string())
    })
}

// ── resolution ────────────────────────────────────────────────────────────────

/// The bell deep link for this plugin's component - `/apps/<applicationId>` -
/// or `None` when it cannot be resolved.
pub fn resolve_app_link(plugin_name: &str, component_id: &str) -> Option<String> {
    let applications = graphql("query { applications { id contentUrl plugin { name } } }")?
        .get("applications")?
        .clone();
    let id = pick_application(&applications, plugin_name, component_id)?;
    Some(format!("/apps/{id}"))
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A real application id, captured from a running instance's `/apps/...`
    /// URL. It is only ever carried through, never parsed - this pins that.
    const KATASTR_APP_ID: &str = "BlBsdWdpbkttYXJrZXRwbGFjZS1wbHVnaW46bWFya2V0cGxhY2UtZGNlODg2Y2MtMjhkZi0zNzk3LWFkY2UtYTU0OTY2OWE4NDY2OmthdGFzdHILQXBwbGljYXRpb24Ha2F0YXN0cg";
    /// Shape the daemon builds for that app: prefix, plugin node id, component
    /// id, trailing slash.
    const KATASTR_CONTENT_URL: &str =
        "/rest/plugin-components/BlBsdWdpbkttYXJrZXRwbGFjZS1wbHVnaW4/katastr/";

    fn app(plugin: &str, content_url: &str) -> serde_json::Value {
        json!({ "id": KATASTR_APP_ID, "contentUrl": content_url, "plugin": { "name": plugin } })
    }

    #[test]
    fn reads_the_component_id_from_a_content_url() {
        assert_eq!(component_id_of(KATASTR_CONTENT_URL), Some("katastr"));
    }

    #[test]
    fn reads_the_component_id_without_a_trailing_slash() {
        assert_eq!(
            component_id_of("/rest/plugin-components/abc/sledovane-dokumenty"),
            Some("sledovane-dokumenty")
        );
    }

    #[test]
    fn has_no_component_id_for_a_degenerate_url() {
        assert_eq!(component_id_of(""), None);
        assert_eq!(component_id_of("/"), None);
    }

    #[test]
    fn picks_the_application_matching_plugin_and_component() {
        let apps = json!([app("katastr", KATASTR_CONTENT_URL)]);
        assert_eq!(
            pick_application(&apps, "katastr", "katastr"),
            Some(KATASTR_APP_ID.to_string())
        );
    }

    #[test]
    fn does_not_match_a_same_named_component_of_another_plugin() {
        // Same component id, different owning plugin - the plugin name is what
        // keeps two marketplaces' same-named components apart.
        let apps = json!([app("kataster-sk", KATASTR_CONTENT_URL)]);
        assert_eq!(pick_application(&apps, "katastr", "katastr"), None);
    }

    #[test]
    fn picks_the_right_sibling_component_of_one_plugin() {
        let apps = json!([
            app("codexis", "/rest/plugin-components/abc/sledovana-judikatura/"),
            app("codexis", "/rest/plugin-components/abc/sledovane-dokumenty/"),
        ]);
        assert_eq!(
            pick_application(&apps, "codexis", "sledovane-dokumenty"),
            Some(KATASTR_APP_ID.to_string())
        );
        assert_eq!(pick_application(&apps, "codexis", "nope"), None);
    }

    #[test]
    fn returns_none_when_the_app_is_not_installed() {
        assert_eq!(pick_application(&json!([]), "katastr", "katastr"), None);
    }

    #[test]
    fn skips_entries_that_are_missing_fields_instead_of_failing() {
        let apps = json!([
            { "contentUrl": KATASTR_CONTENT_URL, "plugin": { "name": "katastr" } },
            { "id": KATASTR_APP_ID, "plugin": { "name": "katastr" } },
            { "id": KATASTR_APP_ID, "contentUrl": KATASTR_CONTENT_URL },
            "not an object",
            app("katastr", KATASTR_CONTENT_URL),
        ]);
        assert_eq!(
            pick_application(&apps, "katastr", "katastr"),
            Some(KATASTR_APP_ID.to_string())
        );
    }
}
