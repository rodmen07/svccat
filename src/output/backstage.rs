//! Backstage.io `catalog-info.yaml` export.
//!
//! A catalog file is a COMMITTED artifact: users write it into their repo and
//! regenerate it when the service catalog changes. So the bytes this module
//! produces have to depend only on the manifest, never on how a map happened to
//! iterate — otherwise every regeneration reports a diff on services nothing
//! changed about, and a `git diff --exit-code` drift check in CI fires on noise.
//! That is why `CatalogMetadata::annotations` is a `BTreeMap` and not a
//! `HashMap`; see the type's own comment for why alphabetical was chosen over
//! insertion order.

use crate::manifest::Manifest;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CatalogInfo {
    api_version: String,
    kind: String,
    metadata: CatalogMetadata,
    spec: CatalogSpec,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CatalogMetadata {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// Annotation keys, serialized in the map's iteration order.
    ///
    /// `BTreeMap`, so that order is alphabetical and identical on every run.
    /// A `HashMap` here serialized in per-process hash order and made the
    /// exported bytes differ between runs over an unchanged manifest.
    ///
    /// Alphabetical was chosen over insertion order deliberately: the insertion
    /// sequence here is just the order of the `if let` arms in `render_export`
    /// (oncall, path, docs, ci), which encodes nothing a reader could rely on,
    /// and preserving it would cost an `indexmap` dependency to express an
    /// accident. Sorted keys are also what every other YAML tool that rewrites a
    /// catalog file will produce, so a hand-edited file round-trips cleanly.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    annotations: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    links: Vec<CatalogLink>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CatalogLink {
    url: String,
    title: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CatalogSpec {
    #[serde(rename = "type")]
    type_field: String,
    lifecycle: String,
    owner: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(rename = "dependsOn", default, skip_serializing_if = "Vec::is_empty")]
    depends_on: Vec<String>,
}

pub fn render_export(manifest: &Manifest) -> Result<String, serde_yaml::Error> {
    let mut docs = Vec::new();
    for svc in &manifest.services {
        let mut annotations = BTreeMap::new();
        if let Some(ref oncall) = svc.oncall {
            annotations.insert("svccat.io/oncall".to_string(), oncall.clone());
        }
        if let Some(ref path) = svc.path {
            annotations.insert("svccat.io/path".to_string(), path.clone());
        }
        if let Some(ref docs_path) = svc.docs {
            annotations.insert("svccat.io/docs".to_string(), docs_path.clone());
        }
        if let Some(ref ci_path) = svc.ci {
            annotations.insert("svccat.io/ci".to_string(), ci_path.clone());
        }

        let mut tags = svc.tags.clone();
        if let Some(ref lang) = svc.language {
            tags.push(lang.to_lowercase());
        }

        let mut links = Vec::new();
        if let Some(ref url) = svc.url {
            links.push(CatalogLink {
                url: url.clone(),
                title: "Website".to_string(),
            });
        }

        let depends_on = svc
            .depends_on
            .iter()
            .map(|dep| format!("component:{}", dep))
            .collect();

        let info = CatalogInfo {
            api_version: "backstage.io/v1alpha1".to_string(),
            kind: "Component".to_string(),
            metadata: CatalogMetadata {
                name: svc.name.clone(),
                description: svc.role.clone(),
                annotations,
                tags,
                links,
            },
            spec: CatalogSpec {
                type_field: "service".to_string(),
                lifecycle: "production".to_string(),
                owner: svc.team.clone().unwrap_or_else(|| "unknown".to_string()),
                system: svc.platform.clone(),
                depends_on,
            },
        };
        docs.push(info);
    }

    let mut out = String::new();
    for (i, doc) in docs.iter().enumerate() {
        if i > 0 {
            out.push_str("---\n");
        }
        out.push_str(&serde_yaml::to_string(doc)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::render_export;
    use crate::manifest::{Manifest, ServiceEntry};

    /// A service declaring every annotation `render_export` knows how to emit.
    ///
    /// Four keys, so an accidental ordering has 4! = 24 arrangements to land in
    /// and a single agreeing pair of renders is already weak evidence of chance.
    fn fully_annotated_manifest() -> Manifest {
        let mut manifest = Manifest::default();
        manifest.services.push(ServiceEntry {
            name: "auth".to_string(),
            oncall: Some("@sec-team".to_string()),
            path: Some("services/auth".to_string()),
            docs: Some("docs/auth.md".to_string()),
            ci: Some(".github/workflows/auth.yml".to_string()),
            ..Default::default()
        });
        manifest
    }

    /// The `svccat.io/*` annotation keys of a rendered document, in printed order.
    ///
    /// Panics when the document carries fewer than two, because a document with
    /// no annotations block orders nothing: every assertion in this module would
    /// compare two empty lists and pass while proving nothing at all.
    fn annotation_keys(yaml: &str) -> Vec<String> {
        let keys: Vec<String> = yaml
            .lines()
            .map(|l| l.trim())
            .filter(|l| l.starts_with("svccat.io/"))
            .map(|l| {
                l.split(':')
                    .next()
                    .expect("an annotation line is `key: value`")
                    .to_string()
            })
            .collect();

        assert!(
            keys.len() >= 2,
            "expected an annotations block with at least two keys, got {keys:?} from:\n{yaml}"
        );
        keys
    }

    /// Determinism, and nothing else: two renders must agree.
    ///
    /// In-process on purpose, and this is the SHARP probe rather than the weak
    /// one. `RandomState` seeds each new `HashMap` from a per-thread counter, so
    /// two maps built inside one process already hash differently — the
    /// `svccat diff` fix (PR #31) saw its pre-fix code fail on the first repeat
    /// inside a single process. The separate-process sampling lives in
    /// `tests/backstage_export_determinism_tests.rs`, which covers the wiring.
    #[test]
    fn repeated_renders_of_one_manifest_are_byte_identical() {
        let manifest = fully_annotated_manifest();
        let first = render_export(&manifest).unwrap();
        annotation_keys(&first);

        for run in 1..64 {
            let again = render_export(&manifest).unwrap();
            assert_eq!(
                first, again,
                "render {run} differed from render 0 for one unchanged manifest"
            );
        }
    }

    /// Which order, kept apart from THAT there is one (L-072): a change that
    /// makes the order stable but not alphabetical must fail exactly here and
    /// leave the determinism test above green.
    #[test]
    fn annotations_are_emitted_in_alphabetical_key_order() {
        let yaml = render_export(&fully_annotated_manifest()).unwrap();
        assert_eq!(
            annotation_keys(&yaml),
            vec![
                "svccat.io/ci".to_string(),
                "svccat.io/docs".to_string(),
                "svccat.io/oncall".to_string(),
                "svccat.io/path".to_string(),
            ],
            "annotation keys should be sorted, got:\n{yaml}"
        );
    }

    /// Ordering the map must not drop or rename anything: the document still
    /// parses, and every declared annotation is present with its value.
    #[test]
    fn every_declared_annotation_survives_with_its_value() {
        let yaml = render_export(&fully_annotated_manifest()).unwrap();
        let doc: serde_yaml::Value =
            serde_yaml::from_str(&yaml).expect("export must be valid YAML");

        let annotations = doc
            .get("metadata")
            .and_then(|m| m.get("annotations"))
            .expect("metadata.annotations must be present");

        for (key, value) in [
            ("svccat.io/oncall", "@sec-team"),
            ("svccat.io/path", "services/auth"),
            ("svccat.io/docs", "docs/auth.md"),
            ("svccat.io/ci", ".github/workflows/auth.yml"),
        ] {
            assert_eq!(
                annotations.get(key).and_then(|v| v.as_str()),
                Some(value),
                "annotation `{key}` missing or wrong in:\n{yaml}"
            );
        }
    }

    /// A service declaring no annotations still emits no `annotations:` key at
    /// all, so the `skip_serializing_if` swap from `HashMap` to `BTreeMap` did
    /// not quietly start writing an empty map into every catalog file.
    #[test]
    fn a_service_with_no_annotations_emits_no_annotations_key() {
        let mut manifest = Manifest::default();
        manifest.services.push(ServiceEntry {
            name: "bare".to_string(),
            ..Default::default()
        });

        let yaml = render_export(&manifest).unwrap();
        assert!(
            !yaml.contains("annotations"),
            "an unannotated service should emit no annotations block, got:\n{yaml}"
        );
    }

    #[test]
    fn test_backstage_render_export() {
        let mut manifest = Manifest::default();
        manifest.services.push(ServiceEntry {
            name: "auth-service".to_string(),
            language: Some("Rust".to_string()),
            platform: Some("Fly.io".to_string()),
            role: Some("Authentication provider".to_string()),
            url: Some("https://auth.example.com".to_string()),
            team: Some("security".to_string()),
            depends_on: vec!["db".to_string()],
            ..Default::default()
        });

        let yaml = render_export(&manifest).unwrap();
        assert!(yaml.contains("apiVersion: backstage.io/v1alpha1"));
        assert!(yaml.contains("kind: Component"));
        assert!(yaml.contains("name: auth-service"));
        assert!(yaml.contains("description: Authentication provider"));
        assert!(yaml.contains("system: Fly.io"));
        assert!(yaml.contains("dependsOn:\n  - component:db"));
        assert!(yaml.contains("owner: security"));
        assert!(yaml.contains("type: service"));
        assert!(yaml.contains("lifecycle: production"));
        assert!(yaml.contains("url: https://auth.example.com"));
        assert!(yaml.contains("title: Website"));
        assert!(yaml.contains("rust"));
    }
}
