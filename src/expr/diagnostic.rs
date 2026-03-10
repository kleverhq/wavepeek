#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLayer {
    Parse,
    Semantic,
    Runtime,
}

impl DiagnosticLayer {
    fn as_str(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::Semantic => "semantic",
            Self::Runtime => "runtime",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprDiagnostic {
    pub layer: DiagnosticLayer,
    pub code: &'static str,
    pub message: String,
    pub primary_span: Span,
    pub notes: Vec<String>,
}

impl ExprDiagnostic {
    pub fn render(&self, source: &str) -> String {
        let header = format!("{}:{}: {}", self.layer.as_str(), self.code, self.message);
        let location = format!(
            "--> span {}..{}",
            self.primary_span.start, self.primary_span.end
        );
        let excerpt = format!("source: {source}");
        let notes = if self.notes.is_empty() {
            String::new()
        } else {
            self.notes
                .iter()
                .map(|note| format!("note: {note}"))
                .collect::<Vec<_>>()
                .join("\n")
        };

        if notes.is_empty() {
            [header, location, excerpt].join("\n")
        } else {
            [header, location, excerpt, notes].join("\n")
        }
    }
}
