use crate::drift::DriftReport;
use crate::manifest::Manifest;

/// Render drift items from a check as CSV.
///
/// Columns: service, severity, kind, message, detail
pub fn render_check(report: &DriftReport) {
    println!("service,severity,kind,message,detail");
    for item in &report.drifts {
        let kind = serde_json::to_value(&item.kind)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default();
        let severity = serde_json::to_value(&item.severity)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default();
        println!(
            "{},{},{},{},{}",
            csv_field(&item.service),
            severity,
            kind,
            csv_field(&item.message),
            csv_field(item.detail.as_deref().unwrap_or(""))
        );
    }
}

/// Render the service catalog as CSV.
///
/// Columns: name, language, platform, role, url, team, oncall
pub fn render_export(manifest: &Manifest) {
    println!("name,language,platform,role,url,team,oncall");
    for svc in &manifest.services {
        println!(
            "{},{},{},{},{},{},{}",
            csv_field(&svc.name),
            csv_field(svc.language.as_deref().unwrap_or("")),
            csv_field(svc.platform.as_deref().unwrap_or("")),
            csv_field(svc.role.as_deref().unwrap_or("")),
            csv_field(svc.url.as_deref().unwrap_or("")),
            csv_field(svc.team.as_deref().unwrap_or("")),
            csv_field(svc.oncall.as_deref().unwrap_or(""))
        );
    }
}

/// Wrap a field value in double-quotes if it contains a comma, double-quote,
/// or newline, escaping internal double-quotes by doubling them (RFC 4180).
fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::csv_field;

    #[test]
    fn leaves_plain_values_unquoted() {
        assert_eq!(csv_field("service-a"), "service-a");
    }

    #[test]
    fn quotes_and_escapes_special_values() {
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(csv_field("a\"b"), "\"a\"\"b\"");
        assert_eq!(csv_field("a\nb"), "\"a\nb\"");
    }
}
