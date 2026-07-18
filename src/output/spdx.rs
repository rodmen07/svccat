use crate::manifest::Manifest;
use anyhow::Result;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SpdxDocument {
    spdx_version: &'static str,
    data_license: &'static str,
    #[serde(rename = "SPDXID")]
    spdxid: &'static str,
    name: String,
    document_namespace: String,
    creation_info: SpdxCreationInfo,
    document_describes: Vec<String>,
    packages: Vec<SpdxPackage>,
    relationships: Vec<SpdxRelationship>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SpdxCreationInfo {
    created: String,
    creators: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SpdxPackage {
    name: String,
    #[serde(rename = "SPDXID")]
    spdxid: String,
    supplier: String,
    download_location: String,
    files_analyzed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    homepage: Option<String>,
    license_concluded: &'static str,
    license_declared: &'static str,
    copyright_text: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    primary_package_purpose: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    external_refs: Vec<SpdxExternalRef>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SpdxExternalRef {
    reference_category: &'static str,
    reference_type: &'static str,
    reference_locator: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SpdxRelationship {
    spdx_element_id: String,
    relationship_type: &'static str,
    related_spdx_element: String,
}

/// Render the service catalog as an SPDX 2.3 JSON software bill of materials:
/// one package per declared service, `DESCRIBES` coverage for every package,
/// and `DEPENDS_ON` relationships derived from `depends_on` edges.
pub fn render_export(manifest: &Manifest) -> Result<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    render_at(
        manifest,
        now.as_secs(),
        now.subsec_nanos(),
        std::process::id(),
    )
}

/// Deterministic seam: everything time- or process-dependent is passed in so
/// unit tests can pin the output exactly.
fn render_at(manifest: &Manifest, secs: u64, subsec_nanos: u32, pid: u32) -> Result<String> {
    let created = crate::timefmt::iso8601_utc(secs);
    let creators = vec![format!("Tool: svccat-{}", env!("CARGO_PKG_VERSION"))];
    let document_namespace = format!(
        "https://spdx.org/spdxdocs/svccat-catalog-{}-{}-{}-{}",
        sanitize_id_fragment(&manifest.version),
        secs,
        subsec_nanos,
        pid
    );

    let mut used_ids: HashSet<String> = HashSet::new();
    let mut id_by_name: HashMap<String, String> = HashMap::new();
    let mut packages: Vec<SpdxPackage> = Vec::new();

    for svc in &manifest.services {
        let base = format!("SPDXRef-Package-{}", sanitize_id_fragment(&svc.name));
        let mut id = base.clone();
        let mut suffix = 2u64;
        while used_ids.contains(&id) {
            id = format!("{base}-{suffix}");
            suffix += 1;
        }
        used_ids.insert(id.clone());
        id_by_name.insert(svc.name.clone(), id.clone());

        let mut external_refs: Vec<SpdxExternalRef> = Vec::new();
        if let Some(ref platform) = svc.platform {
            external_refs.push(SpdxExternalRef {
                reference_category: "OTHER",
                reference_type: "PLATFORM",
                reference_locator: ref_locator(platform),
            });
        }
        if let Some(ref docs) = svc.docs {
            external_refs.push(SpdxExternalRef {
                reference_category: "OTHER",
                reference_type: "DOCUMENTATION",
                reference_locator: ref_locator(docs),
            });
        }

        packages.push(SpdxPackage {
            name: svc.name.clone(),
            spdxid: id,
            supplier: svc
                .team
                .as_ref()
                .map(|team| format!("Organization: {team}"))
                .unwrap_or_else(|| "NOASSERTION".to_string()),
            download_location: "NOASSERTION".to_string(),
            files_analyzed: false,
            homepage: svc.url.clone(),
            license_concluded: "NOASSERTION",
            license_declared: "NOASSERTION",
            copyright_text: "NOASSERTION",
            description: svc.role.clone(),
            primary_package_purpose: "APPLICATION",
            external_refs,
        });
    }

    let document_describes: Vec<String> = packages.iter().map(|p| p.spdxid.clone()).collect();

    let mut relationships: Vec<SpdxRelationship> = document_describes
        .iter()
        .map(|pkg_id| SpdxRelationship {
            spdx_element_id: "SPDXRef-DOCUMENT".to_string(),
            relationship_type: "DESCRIBES",
            related_spdx_element: pkg_id.clone(),
        })
        .collect();

    for (svc, pkg) in manifest.services.iter().zip(&packages) {
        for dep in &svc.depends_on {
            // Unresolved dependency names are skipped: a dangling SPDXID would
            // fail validation, and `svccat deps` already flags them.
            if let Some(dep_id) = id_by_name.get(dep) {
                relationships.push(SpdxRelationship {
                    spdx_element_id: pkg.spdxid.clone(),
                    relationship_type: "DEPENDS_ON",
                    related_spdx_element: dep_id.clone(),
                });
            }
        }
    }

    let doc = SpdxDocument {
        spdx_version: "SPDX-2.3",
        data_license: "CC0-1.0",
        spdxid: "SPDXRef-DOCUMENT",
        name: format!("svccat-catalog-{}", manifest.version),
        document_namespace,
        creation_info: SpdxCreationInfo { created, creators },
        document_describes,
        packages,
        relationships,
    };

    Ok(serde_json::to_string_pretty(&doc)?)
}

/// Map every character outside `[A-Za-z0-9.-]` to `-` so the result is a
/// valid SPDXID fragment. Falls back to `service` for an empty input.
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

/// Percent-encode the characters that would make an SPDX referenceLocator
/// invalid (it must contain no spaces). `%` first so encodings stay unambiguous.
fn ref_locator(raw: &str) -> String {
    raw.replace('%', "%25").replace(' ', "%20")
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

        assert_eq!(v["spdxVersion"], "SPDX-2.3");
        assert_eq!(v["dataLicense"], "CC0-1.0");
        assert_eq!(v["SPDXID"], "SPDXRef-DOCUMENT");
        assert_eq!(v["name"], format!("svccat-catalog-{}", manifest.version));

        let created = v["creationInfo"]["created"].as_str().unwrap();
        let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$").unwrap();
        assert!(re.is_match(created), "not ISO 8601 UTC: {created}");
        assert_eq!(created, "2024-02-29T00:00:00Z");

        let creators = v["creationInfo"]["creators"].as_array().unwrap();
        assert_eq!(
            creators,
            &[Value::from(format!(
                "Tool: svccat-{}",
                env!("CARGO_PKG_VERSION")
            ))]
        );
    }

    #[test]
    fn document_namespace_is_unique_and_uri_safe() {
        let manifest = Manifest::default();
        let v = render_value(&manifest);
        let ns = v["documentNamespace"].as_str().unwrap();
        assert_eq!(
            ns,
            format!(
                "https://spdx.org/spdxdocs/svccat-catalog-{}-100-7-42",
                sanitize_id_fragment(&manifest.version)
            )
        );
        assert!(!ns.contains('#'));
        assert!(!ns.contains(' '));

        // The public wall-clock entry point uses the same prefix.
        for _ in 0..2 {
            let out = render_export(&manifest).unwrap();
            let v: Value = serde_json::from_str(&out).unwrap();
            let ns = v["documentNamespace"].as_str().unwrap();
            assert!(ns.starts_with("https://spdx.org/spdxdocs/svccat-catalog-"));
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
            "spdxVersion",
            "dataLicense",
            "SPDXID",
            "documentNamespace",
            "creationInfo",
            "documentDescribes",
            "packages",
            "relationships",
            "filesAnalyzed",
            "downloadLocation",
            "licenseConcluded",
            "licenseDeclared",
            "copyrightText",
            "primaryPackagePurpose",
            "externalRefs",
            "spdxElementId",
            "relationshipType",
            "relatedSpdxElement",
        ] {
            assert!(out.contains(&format!("\"{key}\"")), "missing key {key}");
        }
        for leaked in [
            "download_location",
            "spdx_version",
            "creation_info",
            "spdx_element_id",
        ] {
            assert!(!out.contains(leaked), "snake_case leakage: {leaked}");
        }
    }

    #[test]
    fn spdxid_sanitization_and_collisions() {
        let mut manifest = Manifest::default();
        manifest.services.push(svc("My Auth_Svc!"));
        manifest.services.push(svc("auth service"));
        manifest.services.push(svc("auth-service"));
        manifest.services.push(svc(""));

        let v = render_value(&manifest);
        let ids: Vec<&str> = v["packages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["SPDXID"].as_str().unwrap())
            .collect();
        assert_eq!(
            ids,
            [
                "SPDXRef-Package-My-Auth-Svc-",
                "SPDXRef-Package-auth-service",
                "SPDXRef-Package-auth-service-2",
                "SPDXRef-Package-service",
            ]
        );
        let re = regex::Regex::new(r"^SPDXRef-Package-[A-Za-z0-9.-]+$").unwrap();
        for id in &ids {
            assert!(re.is_match(id), "invalid SPDXID: {id}");
        }
    }

    #[test]
    fn describes_covers_every_package() {
        let mut manifest = Manifest::default();
        manifest.services.push(svc("a"));
        manifest.services.push(svc("b"));

        let v = render_value(&manifest);
        let pkg_ids: Vec<Value> = v["packages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["SPDXID"].clone())
            .collect();
        assert_eq!(v["documentDescribes"].as_array().unwrap(), &pkg_ids);

        let describes: Vec<&Value> = v["relationships"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| {
                r["relationshipType"] == "DESCRIBES" && r["spdxElementId"] == "SPDXRef-DOCUMENT"
            })
            .collect();
        assert_eq!(describes.len(), pkg_ids.len());
    }

    #[test]
    fn empty_manifest_keeps_arrays_present() {
        let v = render_value(&Manifest::default());
        assert_eq!(v["documentDescribes"].as_array().unwrap().len(), 0);
        assert_eq!(v["packages"].as_array().unwrap().len(), 0);
        assert_eq!(v["relationships"].as_array().unwrap().len(), 0);
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
        let depends: Vec<&Value> = v["relationships"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["relationshipType"] == "DEPENDS_ON")
            .collect();
        assert_eq!(depends.len(), 1);
        assert_eq!(depends[0]["spdxElementId"], "SPDXRef-Package-web");
        assert_eq!(
            depends[0]["relatedSpdxElement"],
            "SPDXRef-Package-declared-dep"
        );
    }

    #[test]
    fn package_field_mapping() {
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
        let pkgs = v["packages"].as_array().unwrap();

        let with_team = &pkgs[0];
        assert_eq!(with_team["supplier"], "Organization: platform");
        assert_eq!(with_team["homepage"], "https://svc.example.com");
        assert_eq!(with_team["downloadLocation"], "NOASSERTION");
        assert_eq!(with_team["licenseConcluded"], "NOASSERTION");
        assert_eq!(with_team["licenseDeclared"], "NOASSERTION");
        assert_eq!(with_team["copyrightText"], "NOASSERTION");
        assert_eq!(with_team["primaryPackagePurpose"], "APPLICATION");

        let refs = with_team["externalRefs"].as_array().unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0]["referenceCategory"], "OTHER");
        assert_eq!(refs[0]["referenceType"], "PLATFORM");
        assert_eq!(refs[0]["referenceLocator"], "Cloud%20Run");
        assert!(!v.to_string().contains("\"TEAM\""));

        let bare = &pkgs[1];
        assert_eq!(bare["supplier"], "NOASSERTION");
        assert!(bare.get("homepage").is_none());
        assert_eq!(bare["downloadLocation"], "NOASSERTION");
    }
}
