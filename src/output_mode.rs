#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Json,
    Jsonl,
}

impl OutputMode {
    pub const fn from_json_flags(json: bool, jsonl: bool) -> Self {
        if jsonl {
            Self::Jsonl
        } else if json {
            Self::Json
        } else {
            Self::Human
        }
    }
}
