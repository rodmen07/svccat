use crate::manifest::Manifest;
use serde::Serialize;
use std::collections::HashMap;

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
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    annotations: HashMap<String, String>,
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
        let mut annotations = HashMap::new();
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
