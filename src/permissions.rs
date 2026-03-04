fn match_segment(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return value.starts_with(prefix);
    }
    pattern == value
}

pub fn match_permission(pattern: &str, resource: &str) -> bool {
    let p: Vec<&str> = pattern.split(':').collect();
    let r: Vec<&str> = resource.split(':').collect();
    if p.len() != r.len() {
        return false;
    }
    p.iter().zip(r.iter()).all(|(ps, rs)| match_segment(ps, rs))
}

pub fn match_any_permission(patterns: &[String], resource: &str) -> bool {
    patterns.iter().any(|p| match_permission(p, resource))
}
