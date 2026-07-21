//! Branded HTML e-mail shell shared by the watcher plugins. The markup and CSS
//! are identical across every app's notification e-mail; only the eyebrow brand
//! label and the footer sentence differ, so those are parameters. Wrap your
//! per-app change-summary HTML with this for a consistent-looking e-mail.

/// Wrap `inner_html` in the standard CODEXIS notification e-mail shell.
///
/// - `eyebrow` - small uppercase brand label (e.g. "Hlídač katastrálních řízení")
/// - `heading` - the H2 title
/// - `inner_html` - the per-app body (already HTML-escaped by the caller)
/// - `footer` - the closing sentence
pub fn email_shell(eyebrow: &str, heading: &str, inner_html: &str, footer: &str) -> String {
    format!(
        "<div style=\"max-width:560px;font:14px sans-serif;color:#0a0a0a\">\
         <p style=\"margin:0;font:600 12px sans-serif;letter-spacing:.14em;\
         text-transform:uppercase;color:#5ea500\">{eyebrow}</p>\
         <h2 style=\"margin:4px 0 2px;font:400 22px Georgia,serif\">{heading}</h2>\
         {inner_html}\
         <p style=\"margin-top:20px;font:13px sans-serif;color:#777\">{footer}</p></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_inner_with_eyebrow_heading_and_footer() {
        let out = email_shell("BRAND", "Nadpis", "<p>obsah</p>", "Patička.");
        assert!(out.contains("BRAND"), "eyebrow: {out}");
        assert!(out.contains("Nadpis"), "heading: {out}");
        assert!(out.contains("<p>obsah</p>"), "inner: {out}");
        assert!(out.contains("Patička."), "footer: {out}");
        // The brand eyebrow keeps the CODEXIS green accent.
        assert!(out.contains("color:#5ea500"), "accent: {out}");
        assert!(
            out.starts_with("<div style=\"max-width:560px"),
            "shell start: {out}"
        );
        assert!(out.trim_end().ends_with("</div>"), "shell end: {out}");
    }
}
