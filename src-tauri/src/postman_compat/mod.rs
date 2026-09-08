pub mod exporter;
pub mod importer;
pub mod local_workspace_importer;
pub mod schema;

#[cfg(test)]
mod tests;

/// Postman stores variables as an ordered *list*, not a map, and real exports routinely define
/// the same key more than once — typically an old value left sitting disabled above the live
/// one. Postman itself folds that list into a map when resolving `{{key}}`, so the **last**
/// definition is the one that's actually in effect. Importing them in order and letting the
/// variable store reject the duplicates does the opposite: it keeps the *first*, i.e. exactly
/// the stale or switched-off value, and discards the real one.
///
/// Collapses `items` to one entry per key, keeping each key's last-defined payload at the
/// position it was first introduced (so display order still matches the source file).
pub(crate) fn dedupe_keys_last_wins<T>(items: impl IntoIterator<Item = (String, T)>) -> Vec<(String, T)> {
    let mut out: Vec<(String, T)> = Vec::new();
    for (key, payload) in items {
        match out.iter_mut().find(|(existing, _)| *existing == key) {
            Some(slot) => slot.1 = payload,
            None => out.push((key, payload)),
        }
    }
    out
}

/// Postman stores a multipart file part's local path as `src`, and on Windows that's sometimes
/// written as `/C:/Users/...` — a POSIX-style leading slash in front of a drive letter, which is
/// how Postman itself normalizes paths internally. Windows file APIs don't accept that shape, so
/// a file that's still sitting right there on disk reads as unopenable both at send time
/// (`http_engine`'s `tokio::fs::read`) and in the editable file-path field the sidebar shows for
/// it. Only that one shape is rewritten (leading slash + single letter + `:` + separator);
/// anything else — a real POSIX path, a relative path, an already-Windows path — passes through
/// unchanged.
pub(crate) fn normalize_local_file_src(src: &str) -> String {
    let bytes = src.as_bytes();
    let is_slash_prefixed_drive_letter = bytes.len() > 3
        && bytes[0] == b'/'
        && bytes[1].is_ascii_alphabetic()
        && bytes[2] == b':'
        && (bytes[3] == b'/' || bytes[3] == b'\\');
    if is_slash_prefixed_drive_letter {
        src[1..].replace('/', "\\")
    } else {
        src.to_string()
    }
}

pub use exporter::{export_environment, export_project_collection};
pub use importer::{
    import_collection, import_environment, CollectionImportReport, EnvironmentImportReport,
};
pub use local_workspace_importer::{import_local_workspace, LocalWorkspaceImportReport};
