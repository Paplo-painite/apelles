//! File I/O shared by the MCP server and the CLI: open/save any supported
//! format, with `.pcraft` (ZIP or directory bundle) handled natively.

use std::path::Path;

use apelles_doc::Document;
use apelles_format::{PcraftWriter, SaveOptions};
use apelles_io::ExportOptions;

use crate::AutomationError;

/// Result of opening a file.
pub struct Opened {
    pub document: Document,
    pub warnings: Vec<String>,
    pub source_read_only: bool,
}

/// Decode a document that was read through a filesystem capability.
pub fn open_bytes(name: &str, bytes: &[u8]) -> Result<Opened, AutomationError> {
    if Path::new(name).extension().is_some_and(|extension| extension.eq_ignore_ascii_case(apelles_format::EXTENSION)) {
        return Ok(Opened { document: apelles_format::load_from_bytes(bytes)?, warnings: Vec::new(), source_read_only: false });
    }
    let r = apelles_io::import(name, bytes)?;
    Ok(Opened { document: r.document, warnings: r.warnings, source_read_only: r.source_read_only })
}

/// Open a document from disk. Directory bundles and `.pcraft` ZIPs load
/// natively; everything else goes through `apelles-io` (PSD, PNG, …).
pub fn open(path: &Path) -> Result<Opened, AutomationError> {
    if path.is_dir() {
        return Ok(Opened { document: apelles_format::load_path(path)?, warnings: Vec::new(), source_read_only: false });
    }
    let bytes = apelles_format::read_file(path).map_err(|e| AutomationError::Io(format!("{}: {e}", path.display())))?;
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    open_bytes(&name, &bytes)
}

/// Previews for a `.pcraft` bundle.
pub fn previews(doc: &Document) -> SaveOptions {
    SaveOptions { thumbnail: Some(apelles_compose::thumbnail(doc, 256)), composite: Some(apelles_compose::thumbnail(doc, 1024)) }
}

/// Save or export `doc` to `path`, choosing the format from the extension
/// (or `format_override`, an extension such as `"png"`). `writer` makes
/// repeated `.pcraft` saves incremental. Returns warnings about lost data.
pub fn save(
    doc: &Document,
    path: &Path,
    format_override: Option<&str>,
    opts: &ExportOptions,
    writer: Option<&mut PcraftWriter>,
) -> Result<Vec<String>, AutomationError> {
    let ext = format_override
        .map(|f| f.trim_start_matches('.').to_ascii_lowercase())
        .or_else(|| path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()))
        .ok_or_else(|| AutomationError::BadRequest(format!("cannot tell the format of `{}`; pass a format", path.display())))?;
    if ext == apelles_format::EXTENSION {
        let mut local = PcraftWriter::new();
        let w = writer.unwrap_or(&mut local);
        w.save_path(doc, path, &previews(doc))?;
        return Ok(Vec::new());
    }
    let r = apelles_io::export(doc, &ext, opts)?;
    // Crash-safe: a failed write never destroys the previous file.
    apelles_format::atomic_write(path, &r.bytes).map_err(|e| AutomationError::Io(e.to_string()))?;
    Ok(r.warnings)
}

/// Encode a document for a capability-scoped write. Unlike [`save`], this
/// never obtains ambient filesystem authority and emits `.pcraft` as a ZIP.
pub fn save_bytes(doc: &Document, name: &str, format_override: Option<&str>, opts: &ExportOptions) -> Result<(Vec<u8>, Vec<String>), AutomationError> {
    let ext = format_override
        .map(|format| format.trim_start_matches('.').to_ascii_lowercase())
        .or_else(|| Path::new(name).extension().map(|extension| extension.to_string_lossy().to_ascii_lowercase()))
        .ok_or_else(|| AutomationError::BadRequest(format!("cannot tell the format of `{name}`; pass a format")))?;
    if ext == apelles_format::EXTENSION {
        return Ok((apelles_format::save_to_bytes(doc, &previews(doc))?, Vec::new()));
    }
    let result = apelles_io::export(doc, &ext, opts)?;
    Ok((result.bytes, result.warnings))
}

/// Flattened document as PNG, scaled to fit `max_side` (0 = full size).
pub fn render_png(doc: &Document, max_side: u32) -> Result<Vec<u8>, AutomationError> {
    let side = if max_side == 0 { doc.size.width.max(doc.size.height) } else { max_side };
    let img = apelles_compose::thumbnail(doc, side.max(1));
    let image = apelles_codecs::Image::from_u8(img.width, img.height, apelles_codecs::ChannelLayout::Rgba, img.pixels)
        .map_err(|e| AutomationError::Other(e.to_string()))?;
    apelles_codecs::encode(&image, apelles_codecs::Format::Png, &Default::default()).map_err(|e| AutomationError::Other(e.to_string()))
}
