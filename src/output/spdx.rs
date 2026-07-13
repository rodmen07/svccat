use crate::drift::DriftReport;
use crate::manifest::Manifest;
use anyhow::Result;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct SpdxDocument {
    spdxVersion: &'static str,
    dataLicense: &'static str,
    SPDXID: &'static str,
    name: String,
    documentNamespace: String,
    creationInfo: SpdxCreationInfo,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    packages: Vec<SpdxPackage>,
}

#[derive(Serialize)]
struct SpdxCreationInfo {
    created: String,
    creators: Vec<String>,
}

#[derive(Serialize)]
struct SpdxPackage {
    name: String,
    SPDXID: String,
    filesAnalyzed: bool,
    downloadLocation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    externalRefs: Vec<SpdxExternalRef>,
}

#[derive(Serialize)]
struct SpdxExternalRef {
    referenceCategory: String,
    referenceType: String,
    referenceLocator: String,
}

/// Render a minimal SPDX 2.3 JSON document representing the service catalog.
/// This initial implementation focuses on JSON output. XML support is planned.
pub fn render_export(manifest: &Manifest, _report: &DriftReport) -> Result<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let created = format!("{}", now.as_secs());

    let creators = vec![format!("Tool: svccat {}", env!("CARGO_PKG_VERSION"))];

    let mut packages: Vec<SpdxPackage> = Vec::new();
    for svc in &manifest.services {
        let id = format!("SPDXRef-Package-{}", svc.name.replace(' ', "_"));
        let download = svc
            .url
            .clone()
            .unwrap_or_else(|| "NOASSERTION".to_string());

        let mut external_refs: Vec<SpdxExternalRef> = Vec::new();
        if let Some(ref platform) = svc.platform {
            external_refs.push(SpdxExternalRef {
                referenceCategory: "OTHER".to_string(),
                referenceType: "PLATFORM".to_string(),
                referenceLocator: platform.clone(),
            });
        }
        if let Some(ref team) = svc.team {
            external_refs.push(SpdxExternalRef {
                referenceCategory: "OTHER".to_string(),
                referenceType: "TEAM".to_string(),
                referenceLocator: team.clone(),
            });
        }
        if let Some(ref docs) = svc.docs {
            external_refs.push(SpdxExternalRef {
                referenceCategory: "OTHER".to_string(),
                referenceType: "DOCUMENTATION".to_string(),
                referenceLocator: docs.clone(),
            });
        }

        let p = SpdxPackage {
            name: svc.name.clone(),
            SPDXID: id,
            filesAnalyzed: false,
            downloadLocation: download,
            description: svc.role.clone(),
            externalRefs: external_refs,
        };
        packages.push(p);
    }

    let doc = SpdxDocument {
        spdxVersion: "SPDX-2.3",
        dataLicense: "CC0-1.0",
        SPDXID: "SPDXRef-DOCUMENT",
        name: format!("svccat catalog snapshot - {}", manifest.version),
        documentNamespace: format!("https://svccat/namespace/{}", created),
        creationInfo: SpdxCreationInfo { created, creators },
        packages,
    };

    Ok(serde_json::to_string_pretty(&doc)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Manifest, ServiceEntry};
    use crate::drift::DriftReport;

    #[test]
    fn test_spdx_render_export_minimal() {
        let mut manifest = Manifest::default();
        manifest.services.push(ServiceEntry {
            name: "auth-service".to_string(),
            language: Some("Rust".to_string()),
            platform: Some("k8s".to_string()),
            role: Some("Auth".to_string()),
            url: Some("https://auth.example.com".to_string()),
            team: Some("security".to_string()),
            ..Default::default()
        });
        let report = DriftReport::default();
        let out = render_export(&manifest, &report).unwrap();
        assert!(out.contains("SPDX-2.3"));
        assert!(out.contains("auth-service"));
    }
}
