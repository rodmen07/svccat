use crate::manifest::Manifest;
use crate::timefmt;
use anyhow::Result;
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// The CycloneDX JSON schema version this exporter targets. 1.7 is the
/// newest full schema published under `CycloneDX/specification` (1.7.1 is
/// an errata-only patch of the same schema, the same way SPDX's exporter
/// emits `SPDX-2.3` without a patch suffix): confirmed via the GitHub
/// releases API (`1.7.1`/`1.6.2`/`1.5.1` are all non-draft, non-prerelease
/// patch tags of their `.0` schema) and by fetching
/// `schema/bom-1.7.schema.json` directly from that repo, which is the
/// schema this module's shape was built and hand-validated against.
const SPEC_VERSION: &str = "1.7";
const SCHEMA_URL: &str = "http://cyclonedx.org/schema/bom-1.7.schema.json";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CycloneDxDocument {
    #[serde(rename = "$schema")]
    schema: &'static str,
    bom_format: &'static str,
    spec_version: &'static str,
    serial_number: String,
    version: u32,
    metadata: CdxMetadata,
    components: Vec<CdxComponent>,
    dependencies: Vec<CdxDependency>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    properties: Vec<CdxProperty>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CdxMetadata {
    timestamp: String,
    tools: CdxTools,
}

/// CycloneDX 1.5+ `metadata.tools` object shape (the flat tool-array form
/// is deprecated). Only `components` is populated: svccat identifies
/// itself the same way SPDX's `creators` array does.
#[derive(Serialize)]
struct CdxTools {
    components: Vec<CdxToolComponent>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CdxToolComponent {
    #[serde(rename = "type")]
    type_field: &'static str,
    name: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CdxComponent {
    #[serde(rename = "type")]
    type_field: &'static str,
    #[serde(rename = "bom-ref")]
    bom_ref: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supplier: Option<CdxOrganizationalEntity>,
    purl: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    external_references: Vec<CdxExternalReference>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    properties: Vec<CdxProperty>,
}

#[derive(Serialize)]
struct CdxOrganizationalEntity {
    name: String,
}

#[derive(Serialize)]
struct CdxExternalReference {
    #[serde(rename = "type")]
    type_field: &'static str,
    url: String,
}

/// Generic name-value extension point (CycloneDX's `properties` array).
/// Used for catalog fields that have no first-class CycloneDX slot, the
/// same role SPDX fills by stretching an `OTHER` external ref -- here we
/// use the mechanism CycloneDX actually provides for that instead.
#[derive(Serialize)]
struct CdxProperty {
    name: String,
    value: String,
}

#[derive(Serialize)]
struct CdxDependency {
    #[serde(rename = "ref")]
    bom_ref: String,
    #[serde(rename = "dependsOn", skip_serializing_if = "Vec::is_empty")]
    depends_on: Vec<String>,
}

/// Render the service catalog as a CycloneDX JSON software bill of
/// materials: one `application` component per declared service, and a
/// `dependencies` graph entry for every component (including
/// dependency-free ones, declared explicitly per the CycloneDX spec's own
/// recommendation rather than omitted) built from `depends_on` edges.
pub fn render_export(manifest: &Manifest) -> Result<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    render_at(
        manifest,
        now.as_secs(),
        now.subsec_nanos(),
        std::process::id(),
    )
}

/// Deterministic seam: everything time- or process-dependent is passed in
/// so unit tests can pin the output exactly (mirrors `spdx::render_at`).
fn render_at(manifest: &Manifest, secs: u64, subsec_nanos: u32, pid: u32) -> Result<String> {
    let timestamp = timefmt::iso8601_utc(secs);

    // Same uniqueness ingredients as SPDX's `documentNamespace`: catalog
    // version, wall-clock time, and process id.
    let seed = format!(
        "svccat-catalog-{}-{}-{}-{}",
        manifest.version, secs, subsec_nanos, pid
    );
    let serial_number = format!("urn:uuid:{}", synthetic_uuid(&seed));

    let mut used_ids: HashSet<String> = HashSet::new();
    let mut bom_ref_by_name: HashMap<String, String> = HashMap::new();
    let mut components: Vec<CdxComponent> = Vec::new();

    for svc in &manifest.services {
        let base = format!("component-{}", sanitize_id_fragment(&svc.name));
        let mut bom_ref = base.clone();
        let mut suffix = 2u64;
        while used_ids.contains(&bom_ref) {
            bom_ref = format!("{base}-{suffix}");
            suffix += 1;
        }
        used_ids.insert(bom_ref.clone());
        bom_ref_by_name.insert(svc.name.clone(), bom_ref.clone());

        let mut external_references: Vec<CdxExternalReference> = Vec::new();
        if let Some(ref url) = svc.url {
            external_references.push(CdxExternalReference {
                type_field: "website",
                url: url.clone(),
            });
        }
        if let Some(ref docs) = svc.docs {
            external_references.push(CdxExternalReference {
                type_field: "documentation",
                url: docs.clone(),
            });
        }

        // `platform` is not a URL, so (unlike SPDX, which stretches an
        // `OTHER` external ref to carry it) it goes in `properties`,
        // CycloneDX's own extension slot for data with no dedicated field.
        let mut properties: Vec<CdxProperty> = Vec::new();
        if let Some(ref platform) = svc.platform {
            properties.push(CdxProperty {
                name: "svccat:platform".to_string(),
                value: platform.clone(),
            });
        }

        components.push(CdxComponent {
            type_field: "application",
            bom_ref,
            name: svc.name.clone(),
            description: svc.role.clone(),
            supplier: svc
                .team
                .as_ref()
                .map(|team| CdxOrganizationalEntity { name: team.clone() }),
            purl: format!("pkg:generic/{}", purl_name(&svc.name)),
            external_references,
            properties,
        });
    }

    let dependencies: Vec<CdxDependency> = manifest
        .services
        .iter()
        .map(|svc| {
            let bom_ref = bom_ref_by_name.get(&svc.name).cloned().unwrap_or_default();
            let depends_on: Vec<String> = svc
                .depends_on
                .iter()
                // Unresolved dependency names are skipped: a dangling ref
                // would fail validation, and `svccat deps` already flags
                // them (same policy as the SPDX exporter's DEPENDS_ON).
                .filter_map(|dep| bom_ref_by_name.get(dep).cloned())
                .collect();
            CdxDependency {
                bom_ref,
                depends_on,
            }
        })
        .collect();

    let doc = CycloneDxDocument {
        schema: SCHEMA_URL,
        bom_format: "CycloneDX",
        spec_version: SPEC_VERSION,
        serial_number,
        version: 1,
        metadata: CdxMetadata {
            timestamp,
            tools: CdxTools {
                components: vec![CdxToolComponent {
                    type_field: "application",
                    name: "svccat",
                    version: env!("CARGO_PKG_VERSION"),
                }],
            },
        },
        components,
        dependencies,
        properties: vec![CdxProperty {
            name: "svccat:manifestVersion".to_string(),
            value: manifest.version.clone(),
        }],
    };

    Ok(serde_json::to_string_pretty(&doc)?)
}

/// Map every character outside `[A-Za-z0-9.-]` to `-` so the result is a
/// readable `bom-ref` fragment. Falls back to `service` for an empty
/// input. CycloneDX's `bom-ref` type itself accepts any non-empty string;
/// this sanitization is for readability, not schema validity.
fn sanitize_id_fragment(raw: &str) -> String {
    let sanitized: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "service".to_string()
    } else {
        sanitized
    }
}

/// Percent-encode a service name into a purl-safe path segment: purl-spec
/// unreserved characters pass through unchanged, every other byte
/// (including each byte of a multi-byte UTF-8 character) is
/// percent-encoded. Falls back to `service` for an empty input, matching
/// `sanitize_id_fragment` and keeping `pkg:generic/<name>` a non-empty,
/// structurally valid purl.
fn purl_name(raw: &str) -> String {
    if raw.is_empty() {
        return "service".to_string();
    }
    let mut out = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        let c = byte as char;
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~') {
            out.push(c);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// Build a deterministic, RFC 4122-shaped (version 4, variant 1) UUID from
/// `seed` using only `std::hash` -- no `uuid` or `rand` dependency needed.
/// Two `DefaultHasher` runs over the same seed with a different trailing
/// tag byte produce the high and low 64 bits; the version and variant
/// nibbles are then forced so the result matches CycloneDX's
/// `serialNumber` pattern exactly. `DefaultHasher::new()` uses fixed keys
/// (SipHash-1-3 with key `(0, 0)`), so this is reproducible across runs
/// and platforms for the same input, exactly like the SPDX exporter's
/// timestamp/pid determinism seam.
fn synthetic_uuid(seed: &str) -> String {
    let hash_tagged = |tag: u8| -> u64 {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        tag.hash(&mut hasher);
        hasher.finish()
    };

    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&hash_tagged(0).to_be_bytes());
    bytes[8..].copy_from_slice(&hash_tagged(1).to_be_bytes());
    bytes[6] = (bytes[6] & 0x0F) | 0x40; // version 4
    bytes[8] = (bytes[8] & 0x3F) | 0x80; // variant 10xx

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Manifest, ServiceEntry};
    use serde_json::Value;

    fn svc(name: &str) -> ServiceEntry {
        ServiceEntry {
            name: name.to_string(),
            ..Default::default()
        }
    }

    fn render_value(manifest: &Manifest) -> Value {
        let out = render_at(manifest, 100, 7, 42).unwrap();
        serde_json::from_str(&out).unwrap()
    }

    #[test]
    fn document_shape() {
        let mut manifest = Manifest::default();
        manifest.services.push(svc("auth-service"));

        let out = render_at(&manifest, 1709164800, 0, 1).unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();

        assert_eq!(
            v["$schema"],
            "http://cyclonedx.org/schema/bom-1.7.schema.json"
        );
        assert_eq!(v["bomFormat"], "CycloneDX");
        assert_eq!(v["specVersion"], "1.7");
        assert_eq!(v["version"], 1);

        let timestamp = v["metadata"]["timestamp"].as_str().unwrap();
        let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$").unwrap();
        assert!(re.is_match(timestamp), "not ISO 8601 UTC: {timestamp}");
        assert_eq!(timestamp, "2024-02-29T00:00:00Z");

        let tool_components = v["metadata"]["tools"]["components"].as_array().unwrap();
        assert_eq!(tool_components.len(), 1);
        assert_eq!(tool_components[0]["type"], "application");
        assert_eq!(tool_components[0]["name"], "svccat");
        assert_eq!(tool_components[0]["version"], env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn serial_number_is_unique_and_urn_valid() {
        let manifest = Manifest::default();
        let v = render_value(&manifest);
        let serial = v["serialNumber"].as_str().unwrap();

        // Exact pattern from CycloneDX's own bom-1.7.schema.json:
        // `^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$`
        let re = regex::Regex::new(
            r"^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
        )
        .unwrap();
        assert!(re.is_match(serial), "not a valid urn:uuid serial: {serial}");

        // Version 4 / variant 1 nibbles, matching what synthetic_uuid forces.
        // Layout after the "urn:uuid:" prefix (9 chars): 8 hex, '-', 4 hex,
        // '-', 4 hex (version nibble is its first char), '-', 4 hex (variant
        // nibble is its first char), '-', 12 hex. That places the version
        // nibble at prefix+8+1+4+1 = 23 and the variant nibble at
        // prefix+8+1+4+1+4+1 = 28.
        let prefix_len = "urn:uuid:".len();
        let version_idx = prefix_len + 14;
        let variant_idx = prefix_len + 19;
        assert_eq!(&serial[version_idx..version_idx + 1], "4");
        assert!(matches!(
            &serial[variant_idx..variant_idx + 1],
            "8" | "9" | "a" | "b"
        ));

        // Same render_at inputs => byte-identical serial (determinism).
        let v2 = render_value(&manifest);
        assert_eq!(v["serialNumber"], v2["serialNumber"]);

        // Different render_at inputs => different serial (uniqueness).
        let out3 = render_at(&manifest, 101, 7, 42).unwrap();
        let v3: Value = serde_json::from_str(&out3).unwrap();
        assert_ne!(v["serialNumber"], v3["serialNumber"]);

        // The public wall-clock entry point uses the same scheme.
        for _ in 0..2 {
            let out = render_export(&manifest).unwrap();
            let v: Value = serde_json::from_str(&out).unwrap();
            let serial = v["serialNumber"].as_str().unwrap();
            assert!(serial.starts_with("urn:uuid:"));
            assert!(re.is_match(serial));
        }
    }

    #[test]
    fn key_casing_locked_to_camel_case() {
        let mut manifest = Manifest::default();
        manifest.services.push(ServiceEntry {
            name: "auth-service".to_string(),
            platform: Some("Cloud Run".to_string()),
            docs: Some("docs/auth.md".to_string()),
            url: Some("https://auth.example.com".to_string()),
            team: Some("security".to_string()),
            role: Some("Auth".to_string()),
            depends_on: vec!["auth-service".to_string()],
            ..Default::default()
        });

        let out = render_at(&manifest, 100, 7, 42).unwrap();
        for key in [
            "bomFormat",
            "specVersion",
            "serialNumber",
            "\"bom-ref\"",
            "externalReferences",
            "\"ref\"",
            "\"dependsOn\"",
            "\"type\"",
        ] {
            assert!(out.contains(key), "missing key {key}");
        }
        for leaked in [
            "bom_format",
            "spec_version",
            "serial_number",
            "bom_ref",
            "external_references",
            "depends_on",
            "type_field",
        ] {
            assert!(!out.contains(leaked), "snake_case leakage: {leaked}");
        }
    }

    #[test]
    fn bom_ref_sanitization_and_collisions() {
        let mut manifest = Manifest::default();
        manifest.services.push(svc("My Auth_Svc!"));
        manifest.services.push(svc("auth service"));
        manifest.services.push(svc("auth-service"));
        manifest.services.push(svc(""));

        let v = render_value(&manifest);
        let ids: Vec<&str> = v["components"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["bom-ref"].as_str().unwrap())
            .collect();
        assert_eq!(
            ids,
            [
                "component-My-Auth-Svc-",
                "component-auth-service",
                "component-auth-service-2",
                "component-service",
            ]
        );
        let re = regex::Regex::new(r"^component-[A-Za-z0-9.-]+$").unwrap();
        for id in &ids {
            assert!(re.is_match(id), "invalid bom-ref: {id}");
        }
    }

    #[test]
    fn dependencies_cover_every_component() {
        let mut manifest = Manifest::default();
        manifest.services.push(svc("a"));
        manifest.services.push(svc("b"));

        let v = render_value(&manifest);
        let component_refs: Vec<Value> = v["components"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["bom-ref"].clone())
            .collect();
        let dependency_refs: Vec<Value> = v["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d["ref"].clone())
            .collect();
        // Every component gets a dependency-graph entry, dependency-free
        // ones included explicitly, per the CycloneDX spec's own
        // recommendation (mirrors SPDX's DESCRIBES-covers-every-package).
        assert_eq!(dependency_refs, component_refs);
    }

    #[test]
    fn empty_manifest_keeps_arrays_present() {
        let v = render_value(&Manifest::default());
        assert_eq!(v["components"].as_array().unwrap().len(), 0);
        assert_eq!(v["dependencies"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn depends_on_skips_unresolved_names() {
        let mut manifest = Manifest::default();
        manifest.services.push(ServiceEntry {
            name: "web".to_string(),
            depends_on: vec!["declared-dep".to_string(), "ghost".to_string()],
            ..Default::default()
        });
        manifest.services.push(svc("declared-dep"));

        let v = render_value(&manifest);
        let dependencies = v["dependencies"].as_array().unwrap();
        let web_dep = dependencies
            .iter()
            .find(|d| d["ref"] == "component-web")
            .unwrap();
        let depends_on = web_dep["dependsOn"].as_array().unwrap();
        assert_eq!(depends_on.len(), 1);
        assert_eq!(depends_on[0], "component-declared-dep");
    }

    #[test]
    fn component_field_mapping() {
        let mut manifest = Manifest::default();
        manifest.services.push(ServiceEntry {
            name: "with-team".to_string(),
            team: Some("platform".to_string()),
            url: Some("https://svc.example.com".to_string()),
            platform: Some("Cloud Run".to_string()),
            ..Default::default()
        });
        manifest.services.push(svc("bare"));

        let v = render_value(&manifest);
        let components = v["components"].as_array().unwrap();

        let with_team = &components[0];
        assert_eq!(with_team["type"], "application");
        assert_eq!(with_team["supplier"]["name"], "platform");
        assert_eq!(with_team["purl"], "pkg:generic/with-team");

        let refs = with_team["externalReferences"].as_array().unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0]["type"], "website");
        assert_eq!(refs[0]["url"], "https://svc.example.com");

        let props = with_team["properties"].as_array().unwrap();
        assert_eq!(props.len(), 1);
        assert_eq!(props[0]["name"], "svccat:platform");
        assert_eq!(props[0]["value"], "Cloud Run");
        assert!(!v.to_string().contains("\"TEAM\""));

        let bare = &components[1];
        assert!(bare.get("supplier").is_none());
        assert!(bare.get("description").is_none());
        assert_eq!(bare["purl"], "pkg:generic/bare");
        assert!(bare.get("externalReferences").is_none());
        assert!(bare.get("properties").is_none());
    }

    #[test]
    fn purl_percent_encodes_unsafe_characters() {
        assert_eq!(purl_name("auth service"), "auth%20service");
        assert_eq!(purl_name(""), "service");
        assert_eq!(purl_name("auth-service_v2.1~x"), "auth-service_v2.1~x");
        // Multi-byte UTF-8 (e.g. accented/CJK characters) is percent-encoded
        // byte by byte, keeping the purl a valid ASCII-only path segment.
        assert_eq!(purl_name("café"), "caf%C3%A9");
    }

    #[test]
    fn unusual_characters_in_service_name_stay_schema_valid() {
        // A service name that is simultaneously non-ASCII, contains purl- and
        // bom-ref-unsafe characters, and has no depends_on: every derived
        // identifier must still come out well-formed.
        let mut manifest = Manifest::default();
        manifest.services.push(svc("サービス/日本語 Auth_Svc! 🎉"));

        let v = render_value(&manifest);
        let component = &v["components"][0];
        let bom_ref = component["bom-ref"].as_str().unwrap();
        let re = regex::Regex::new(r"^component-[A-Za-z0-9.-]+$").unwrap();
        assert!(re.is_match(bom_ref), "invalid bom-ref: {bom_ref}");

        let purl = component["purl"].as_str().unwrap();
        assert!(purl.starts_with("pkg:generic/"));
        assert!(purl.is_ascii(), "purl must be ASCII-only: {purl}");

        // No depends_on declared: the dependency-graph entry still exists
        // (every component gets one) but carries no dependsOn array.
        let dependency = &v["dependencies"][0];
        assert_eq!(dependency["ref"], component["bom-ref"]);
        assert!(dependency.get("dependsOn").is_none());

        // The original, un-sanitized name is preserved verbatim in `name`.
        assert_eq!(component["name"], "サービス/日本語 Auth_Svc! 🎉");
    }

    #[test]
    fn top_level_property_carries_manifest_version() {
        let manifest = Manifest::default();
        let v = render_value(&manifest);
        let props = v["properties"].as_array().unwrap();
        let manifest_version_prop = props
            .iter()
            .find(|p| p["name"] == "svccat:manifestVersion")
            .unwrap();
        assert_eq!(manifest_version_prop["value"], manifest.version);
    }
}
