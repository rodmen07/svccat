//! Safely embed serializable data as a `<script>` payload in generated HTML.
//!
//! Service names, team names, and drift messages ultimately come from files a
//! repo owner controls (`services.yaml`, `svccat.toml`), so they are untrusted
//! text from the report's point of view: a service named `</script><script>
//! alert(1)</script>` must not become live markup just because it round-tripped
//! through `serde_json`.
//!
//! `serde_json::to_string` already escapes the characters JSON requires
//! (`"`, `\`, and control characters), which is enough to keep the payload
//! valid JSON. It is *not* enough to keep it safe inside an HTML `<script>`
//! element: the HTML tokenizer looks for the literal, case-insensitive byte
//! sequence `</script` inside *any* script element's text, regardless of what
//! a JS or JSON parser would make of it. A JSON string value containing that
//! substring closes the element early, and everything after it is parsed as
//! new markup — a classic stored-HTML-injection vector.
//!
//! [`embed`] closes that gap by additionally escaping `<`, `>`, and `&` to
//! their `\uXXXX` forms after JSON-encoding. Those three characters never
//! appear in JSON's own structural syntax (`{ } [ ] : , "`), so the
//! replacement can only ever land inside a string *value*, never break JSON
//! structure. The result is valid JSON text that is safe to place verbatim
//! inside a `<script>` element, whether that element is read via
//! `JSON.parse(...)` (`type="application/json"`) or inlined directly as a JS
//! literal (`const nodes = <embed output>;`), because `\uXXXX` escapes are
//! valid in both JSON and JS string literals.

use serde::Serialize;

/// Serialize `value` to JSON and neutralize `<`, `>`, and `&` so the result
/// can be embedded verbatim inside an HTML `<script>` element.
pub(crate) fn embed<T: Serialize>(value: &T) -> serde_json::Result<String> {
    let raw = serde_json::to_string(value)?;
    Ok(raw
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_values_round_trip_unchanged_in_meaning() {
        let embedded = embed(&serde_json::json!({"id": "api", "count": 3})).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&embedded).unwrap();
        assert_eq!(parsed["id"], "api");
        assert_eq!(parsed["count"], 3);
    }

    #[test]
    fn script_breakout_attempt_is_neutralized() {
        // The classic payload: a value that would close the surrounding
        // <script> element early if embedded raw.
        let malicious = "</script><script>alert(1)</script>";
        let embedded = embed(&serde_json::json!({ "id": malicious })).unwrap();

        assert!(
            !embedded.contains("</script"),
            "embedded payload must not contain a literal script-close sequence: {embedded}"
        );
        assert!(
            !embedded.to_lowercase().contains("</script"),
            "embedded payload must not contain a case-insensitive script-close sequence: {embedded}"
        );

        // The escaping is reversible: parsing it back as JSON recovers the
        // exact original string, proving nothing was corrupted or dropped,
        // only made inert as markup.
        let parsed: serde_json::Value = serde_json::from_str(&embedded).unwrap();
        assert_eq!(parsed["id"], malicious);
    }

    #[test]
    fn ampersand_and_angle_brackets_are_escaped_everywhere() {
        let embedded = embed(&serde_json::json!(["<b>&</b>"])).unwrap();
        assert!(!embedded.contains('<'));
        assert!(!embedded.contains('>'));
        assert!(!embedded.contains('&'));

        let parsed: serde_json::Value = serde_json::from_str(&embedded).unwrap();
        assert_eq!(parsed[0], "<b>&</b>");
    }

    #[test]
    fn quotes_and_backslashes_stay_valid_json() {
        let embedded = embed(&serde_json::json!({"name": "a\"b\\c"})).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&embedded).unwrap();
        assert_eq!(parsed["name"], "a\"b\\c");
    }
}
