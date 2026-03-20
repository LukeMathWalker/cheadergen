use std::path::PathBuf;

use rustdoc_types::Span;

/// Severity level for a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

/// A source location derived from `rustdoc_types::Span`.
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// Absolute path to the source file (used to read the file).
    pub file_path: PathBuf,
    /// Workspace-relative display path (used in rendered output).
    pub display_path: String,
    /// 1-indexed (line, col) start position.
    pub begin: (usize, usize),
    /// 1-indexed (line, col) end position.
    pub end: (usize, usize),
    /// Optional label for the annotation span.
    pub label: Option<String>,
}

/// A single diagnostic message with optional source location and annotations.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub source_location: Option<SourceLocation>,
    pub help: Option<String>,
    pub notes: Vec<String>,
}

/// Collects diagnostics emitted during analysis.
pub struct DiagnosticSink {
    diagnostics: Vec<Diagnostic>,
    workspace_root: PathBuf,
}

impl DiagnosticSink {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            diagnostics: Vec::new(),
            workspace_root,
        }
    }

    pub fn warning(&mut self, msg: impl Into<String>) -> DiagnosticBuilder<'_> {
        DiagnosticBuilder {
            sink: self,
            diagnostic: Diagnostic {
                severity: Severity::Warning,
                message: msg.into(),
                source_location: None,
                help: None,
                notes: Vec::new(),
            },
        }
    }

    pub fn error(&mut self, msg: impl Into<String>) -> DiagnosticBuilder<'_> {
        DiagnosticBuilder {
            sink: self,
            diagnostic: Diagnostic {
                severity: Severity::Error,
                message: msg.into(),
                source_location: None,
                help: None,
                notes: Vec::new(),
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn drain(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }
}

/// Fluent builder for constructing and emitting a [`Diagnostic`].
pub struct DiagnosticBuilder<'a> {
    sink: &'a mut DiagnosticSink,
    diagnostic: Diagnostic,
}

impl DiagnosticBuilder<'_> {
    /// Attach a source location from a `rustdoc_types::Span`.
    ///
    /// Span filenames from rustdoc are resolved relative to the workspace root
    /// (stored in the sink). Absolute paths (e.g. from registry crates) are
    /// used as-is.
    pub fn with_span(mut self, span: &Span) -> Self {
        let ws_root = &self.sink.workspace_root;
        let file_path = if span.filename.is_absolute() {
            span.filename.clone()
        } else {
            ws_root.join(&span.filename)
        };
        let display_path = pathdiff::diff_paths(&file_path, ws_root)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| file_path.display().to_string());
        self.diagnostic.source_location = Some(SourceLocation {
            file_path,
            display_path,
            begin: span.begin,
            end: span.end,
            label: None,
        });
        self
    }

    /// Attach a source location only if a span is available.
    pub fn with_span_if(self, span: Option<&Span>) -> Self {
        if let Some(span) = span {
            self.with_span(span)
        } else {
            self
        }
    }

    /// Set the annotation label on the source span.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        if let Some(ref mut loc) = self.diagnostic.source_location {
            loc.label = Some(label.into());
        }
        self
    }

    /// Add a help message to the diagnostic.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.diagnostic.help = Some(help.into());
        self
    }

    /// Add a note to the diagnostic.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.diagnostic.notes.push(note.into());
        self
    }

    /// Emit the diagnostic into the sink.
    pub fn emit(self) {
        self.sink.diagnostics.push(self.diagnostic);
    }
}
