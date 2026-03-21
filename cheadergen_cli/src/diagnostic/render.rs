use std::collections::HashMap;
use std::path::PathBuf;

use annotate_snippets::{Level, Renderer, Snippet};

use super::sink::{Diagnostic, Severity, SourceLocation};

/// Render a list of diagnostics to a string suitable for stderr output.
///
/// When `use_color` is true, ANSI color codes are included.
pub fn render_diagnostics(diagnostics: &[Diagnostic], use_color: bool) -> String {
    let renderer = if use_color {
        Renderer::styled()
    } else {
        Renderer::plain()
    };

    let mut file_cache: HashMap<PathBuf, String> = HashMap::new();
    let mut output = String::new();

    for diagnostic in diagnostics {
        let rendered = render_one(diagnostic, &renderer, &mut file_cache);
        if !output.is_empty() {
            output.push_str("\n\n");
        }
        output.push_str(&rendered);
    }

    output
}

fn render_one(
    diagnostic: &Diagnostic,
    renderer: &Renderer,
    file_cache: &mut HashMap<PathBuf, String>,
) -> String {
    let level = match diagnostic.severity {
        Severity::Warning => Level::Warning,
        Severity::Error => Level::Error,
    };

    let mut message = level.title(&diagnostic.message);

    // Temporaries that must outlive the message builder.
    let mut relative_path_str = String::new();
    let mut source_text = String::new();
    let mut annotation_label = String::new();

    if let Some(ref loc) = diagnostic.source_location
        && let Some(snippet) = build_snippet(
            loc,
            level,
            file_cache,
            &mut relative_path_str,
            &mut source_text,
            &mut annotation_label,
        )
    {
        message = message.snippet(snippet);
    }

    // Add notes as footers.
    for note in &diagnostic.notes {
        message = message.footer(Level::Note.title(note));
    }

    // Add error chain as note footers.
    // Temporaries for formatted "caused by" strings must outlive `message`.
    let caused_by: Vec<String> = diagnostic
        .error_chain
        .get(1..)
        .unwrap_or_default()
        .iter()
        .map(|cause| format!("caused by: {cause}"))
        .collect();
    if let Some(first) = diagnostic.error_chain.first() {
        message = message.footer(Level::Note.title(first));
        for text in &caused_by {
            message = message.footer(Level::Note.title(text));
        }
    }

    // Add help as footer.
    if let Some(ref help) = diagnostic.help {
        message = message.footer(Level::Help.title(help));
    }

    renderer.render(message).to_string()
}

fn build_snippet<'a>(
    loc: &SourceLocation,
    level: Level,
    file_cache: &mut HashMap<PathBuf, String>,
    display_path_buf: &'a mut String,
    source_buf: &'a mut String,
    label_buf: &'a mut String,
) -> Option<Snippet<'a>> {
    // Read the source file (cached).
    let source = match file_cache.get(&loc.file_path) {
        Some(s) => s.as_str(),
        None => {
            let content = std::fs::read_to_string(&loc.file_path).ok()?;
            file_cache.insert(loc.file_path.clone(), content);
            file_cache.get(&loc.file_path).unwrap().as_str()
        }
    };

    *display_path_buf = loc.display_path.clone();

    // Convert 1-indexed (line, col) to byte offsets within the source.
    let (byte_start, byte_end, line_start) = line_col_to_byte_range(source, loc.begin, loc.end)?;

    // Extract only the relevant lines for the snippet.
    let line_end = loc.end.0;
    let (snippet_source, snippet_byte_start, snippet_byte_end) =
        extract_lines(source, line_start, line_end, byte_start, byte_end);
    *source_buf = snippet_source;

    let mut snippet = Snippet::source(source_buf)
        .line_start(line_start)
        .origin(display_path_buf)
        .fold(true);

    // Build annotation.
    let mut annotation = level.span(snippet_byte_start..snippet_byte_end);
    if let Some(ref label) = loc.label {
        *label_buf = label.clone();
        annotation = annotation.label(label_buf);
    }
    snippet = snippet.annotation(annotation);

    Some(snippet)
}

/// Convert 1-indexed (line, col) pairs to byte offsets in `source`.
/// Returns `(byte_start, byte_end, start_line_number)`.
fn line_col_to_byte_range(
    source: &str,
    begin: (usize, usize),
    end: (usize, usize),
) -> Option<(usize, usize, usize)> {
    let (begin_line, begin_col) = begin;
    let (end_line, end_col) = end;

    let mut byte_start = None;
    let mut byte_end = None;
    let mut current_byte = 0;

    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1; // 1-indexed

        if line_num == begin_line {
            // Column is 0-indexed in rustdoc spans
            let col_byte = line
                .char_indices()
                .nth(begin_col)
                .map(|(i, _)| i)
                .unwrap_or(line.len());
            byte_start = Some(current_byte + col_byte);
        }

        if line_num == end_line {
            let col_byte = line
                .char_indices()
                .nth(end_col)
                .map(|(i, _)| i)
                .unwrap_or(line.len());
            byte_end = Some(current_byte + col_byte);
            break;
        }

        // +1 for the newline character
        current_byte += line.len() + 1;
    }

    Some((byte_start?, byte_end.unwrap_or(byte_start?), begin_line))
}

/// Extract the source lines from `line_start` to `line_end` (1-indexed, inclusive),
/// and adjust the byte offsets to be relative to the extracted text.
fn extract_lines(
    source: &str,
    line_start: usize,
    line_end: usize,
    abs_byte_start: usize,
    abs_byte_end: usize,
) -> (String, usize, usize) {
    let mut result = String::new();
    let mut snippet_start_byte = None;

    let mut current_byte = 0;
    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;
        if line_num >= line_start && line_num <= line_end {
            if snippet_start_byte.is_none() {
                snippet_start_byte = Some(current_byte);
            }
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
        }
        if line_num > line_end {
            break;
        }
        current_byte += line.len() + 1;
    }

    let base = snippet_start_byte.unwrap_or(0);
    let rel_start = abs_byte_start.saturating_sub(base);
    let rel_end = abs_byte_end.saturating_sub(base);

    (result, rel_start, rel_end)
}
