use std::path::PathBuf;

pub fn propfind_response(href: &str, path: &PathBuf, is_dir: bool) -> String {
    propfind_response_named(href, path, is_dir, None)
}

pub fn propfind_response_named(href: &str, path: &PathBuf, is_dir: bool, display_name: Option<&str>) -> String {
    let displayname = display_name
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| href.trim_start_matches('/').to_string())
        });

    let resource_type = if is_dir { r#"<D:collection/>"# } else { "" };

    let content_length = std::fs::metadata(path)
        .map(|m| m.len().to_string())
        .unwrap_or_else(|_| "0".to_string());

    let rfc1123_fmt = time::format_description::parse(
        "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] GMT",
    )
    .unwrap();

    let last_modified = std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| {
            let datetime: time::OffsetDateTime = t.into();
            datetime
                .to_offset(time::UtcOffset::UTC)
                .format(&rfc1123_fmt)
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let creation_date = std::fs::metadata(path)
        .ok()
        .and_then(|m| m.created().ok())
        .map(|t| {
            let datetime: time::OffsetDateTime = t.into();
            datetime
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let etag = std::fs::metadata(path)
        .ok()
        .map(|m| {
            use std::hash::{Hash, Hasher};
            use std::os::unix::fs::MetadataExt;
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            m.ino().hash(&mut hasher);
            m.mtime().hash(&mut hasher);
            format!("\"{:x}\"", hasher.finish())
        })
        .unwrap_or_default();

    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    format!(
        r#"<D:response>
<D:href>{}</D:href>
<D:propstat>
<D:prop>
<D:displayname>{}</D:displayname>
<D:resource-type>{}</D:resource-type>
<D:getcontentlength>{}</D:getcontentlength>
<D:getcontenttype>{}</D:getcontenttype>
<D:getlastmodified>{}</D:getlastmodified>
<D:creationdate>{}</D:creationdate>
<D:getetag>{}</D:getetag>
</D:prop>
<D:status>HTTP/1.1 200 OK</D:status>
</D:propstat>
</D:response>"#,
        href,
        escape_xml(&displayname),
        resource_type,
        content_length,
        escape_xml(&mime),
        last_modified,
        creation_date,
        etag
    )
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
