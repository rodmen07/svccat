//! Best-effort inference of service metadata fields from the files present in a
//! service directory.
//!
//! Used by `svccat init` and `svccat fix` to pre-populate `language` and
//! `platform` so a freshly scaffolded manifest has fewer `~` placeholders — and
//! therefore less `missing_field` drift — straight out of the box.
//!
//! Inference is deliberately conservative: every function returns `None` unless
//! there is an unambiguous signal on disk. It is better to leave a field as a
//! placeholder than to guess a wrong value the user must later correct.

use std::path::Path;

/// Language inferred from the build / dependency manifest present in a service
/// directory.
///
/// Returns `None` when no recognised build file is found. The first matching
/// candidate in declaration order wins, so higher-signal markers are listed
/// first.
pub fn infer_language(root: &Path, service_path: &str) -> Option<String> {
    let dir = root.join(service_path);

    // (marker file, language). Order matters: the first hit wins.
    const CANDIDATES: &[(&str, &str)] = &[
        ("Cargo.toml", "Rust"),
        ("go.mod", "Go"),
        ("package.json", "TypeScript"),
        ("pyproject.toml", "Python"),
        ("requirements.txt", "Python"),
        ("setup.py", "Python"),
        ("pom.xml", "Java"),
        ("build.gradle", "Java"),
        ("build.gradle.kts", "Kotlin"),
        ("CMakeLists.txt", "C++"),
        ("Directory.Build.props", "C#"),
        ("Gemfile", "Ruby"),
        ("mix.exs", "Elixir"),
        ("pubspec.yaml", "Dart"),
        ("composer.json", "PHP"),
    ];

    for (marker, lang) in CANDIDATES {
        if dir.join(marker).exists() {
            return Some((*lang).to_string());
        }
    }

    None
}

/// Deployment platform inferred from the infrastructure / deploy descriptor
/// files present in a service directory.
///
/// Conservative by design: only unambiguous deploy descriptors map to a
/// platform. A bare `Dockerfile` is intentionally *not* treated as a platform
/// signal — it describes how a service is packaged, not where it runs, and is
/// present in nearly every service directory — so it would only add noise.
///
/// Returns `None` when nothing matches.
pub fn infer_platform(root: &Path, service_path: &str) -> Option<String> {
    let dir = root.join(service_path);

    // (descriptor file, platform). Most specific / highest-signal first.
    const FILE_SIGNALS: &[(&str, &str)] = &[
        ("fly.toml", "fly.io"),
        ("vercel.json", "vercel"),
        ("netlify.toml", "netlify"),
        ("render.yaml", "render"),
        ("railway.json", "railway"),
        ("railway.toml", "railway"),
        ("app.yaml", "gcp-app-engine"),
        ("Procfile", "heroku"),
        ("serverless.yml", "aws-lambda"),
        ("serverless.yaml", "aws-lambda"),
        ("template.yaml", "aws-sam"),
        ("samconfig.toml", "aws-sam"),
        ("cloudbuild.yaml", "gcp-cloud-build"),
        ("Chart.yaml", "kubernetes-helm"),
        ("kustomization.yaml", "kubernetes"),
        ("kustomization.yml", "kubernetes"),
    ];

    for (file, platform) in FILE_SIGNALS {
        if dir.join(file).exists() {
            return Some((*platform).to_string());
        }
    }

    // Directory-level signals (e.g. a `k8s/` or `helm/` folder).
    const DIR_SIGNALS: &[(&str, &str)] = &[
        ("helm", "kubernetes-helm"),
        ("k8s", "kubernetes"),
        ("kubernetes", "kubernetes"),
        (".platform", "aws-elastic-beanstalk"),
    ];

    for (subdir, platform) in DIR_SIGNALS {
        if dir.join(subdir).is_dir() {
            return Some((*platform).to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn touch(root: &Path, rel: &str) {
        let p = root.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&p, "").unwrap();
    }

    #[test]
    fn infers_language_from_build_files() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        touch(root, "svc/Cargo.toml");
        assert_eq!(infer_language(root, "svc").as_deref(), Some("Rust"));
    }

    #[test]
    fn language_is_none_without_markers() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("svc")).unwrap();
        assert_eq!(infer_language(root, "svc"), None);
    }

    #[test]
    fn infers_platform_from_descriptor_file() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        touch(root, "svc/fly.toml");
        assert_eq!(infer_platform(root, "svc").as_deref(), Some("fly.io"));
    }

    #[test]
    fn infers_platform_from_directory_signal() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("svc/k8s")).unwrap();
        assert_eq!(infer_platform(root, "svc").as_deref(), Some("kubernetes"));
    }

    #[test]
    fn dockerfile_alone_is_not_a_platform_signal() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        touch(root, "svc/Dockerfile");
        assert_eq!(infer_platform(root, "svc"), None);
    }

    #[test]
    fn platform_is_none_without_descriptors() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        touch(root, "svc/Cargo.toml");
        assert_eq!(infer_platform(root, "svc"), None);
    }
}
