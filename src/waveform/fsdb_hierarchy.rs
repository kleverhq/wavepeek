#![allow(dead_code)]

use std::collections::HashMap;

use crate::error::WavepeekError;
use crate::expr::{ExprStorage, ExprType, ExprTypeKind, IntegerLikeKind};

use super::types::{
    EXCLUDED_SCOPE_KIND_ALIASES, EXCLUDED_SIGNAL_KIND_ALIASES, ExprResolvedSignal, ResolvedSignal,
    STABLE_SCOPE_KIND_ALIASES, STABLE_SIGNAL_KIND_ALIASES, ScopeEntry, SignalEntry, SignalId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawScopeKind {
    Module,
    Task,
    Function,
    Begin,
    Fork,
    Generate,
    Struct,
    Union,
    Class,
    Interface,
    Package,
    Program,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawSignalKind {
    Event,
    Integer,
    Parameter,
    Real,
    Reg,
    Supply0,
    Supply1,
    Time,
    Tri,
    TriAnd,
    TriOr,
    TriReg,
    Tri0,
    Tri1,
    WAnd,
    Wire,
    WOr,
    String,
    Port,
    SparseArray,
    RealTime,
    RealParameter,
    Bit,
    Logic,
    Int,
    ShortInt,
    LongInt,
    Byte,
    Enum,
    ShortReal,
    Boolean,
    BitVector,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawDatatypeKind {
    Enum,
    Logic,
    Bit,
    Int,
    UInt,
    ShortInt,
    ShortUInt,
    LongInt,
    LongUInt,
    Byte,
    UByte,
    Real,
    ShortReal,
    Time,
    String,
    Event,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawScopeRecord {
    pub(super) name: String,
    pub(super) kind: RawScopeKind,
    pub(super) hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawSignalRecord {
    pub(super) idcode: u64,
    pub(super) name: String,
    pub(super) kind: RawSignalKind,
    pub(super) left: Option<i32>,
    pub(super) right: Option<i32>,
    pub(super) datatype_id: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawDatatypeRecord {
    pub(super) idcode: u16,
    pub(super) kind: RawDatatypeKind,
}

#[derive(Debug)]
pub(super) struct FsdbHierarchyBuilder {
    scopes: Vec<ScopeNode>,
    scope_by_path: HashMap<String, usize>,
    signal_by_path: HashMap<String, usize>,
    signals: Vec<FsdbSignalInfo>,
    roots: Vec<usize>,
    stack: Vec<StackEntry>,
    datatypes: HashMap<u16, RawDatatypeKind>,
}

#[derive(Debug, Clone)]
pub(super) struct FsdbHierarchyIndex {
    scopes: Vec<ScopeNode>,
    signals: Vec<FsdbSignalInfo>,
    roots: Vec<usize>,
    scope_by_path: HashMap<String, usize>,
    signal_by_path: HashMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScopeNode {
    name: String,
    path: String,
    kind: String,
    depth: usize,
    parent: Option<usize>,
    children: Vec<usize>,
    signals: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FsdbSignalInfo {
    name: String,
    path: String,
    kind: String,
    width: Option<u32>,
    idcode: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StackEntry {
    scope_index: Option<usize>,
    hidden: bool,
}

impl FsdbHierarchyBuilder {
    pub(super) fn new() -> Self {
        Self {
            scopes: Vec::new(),
            scope_by_path: HashMap::new(),
            signal_by_path: HashMap::new(),
            signals: Vec::new(),
            roots: Vec::new(),
            stack: Vec::new(),
            datatypes: HashMap::new(),
        }
    }

    pub(super) fn begin_tree(&mut self) {
        self.stack.clear();
    }

    pub(super) fn scope(&mut self, record: RawScopeRecord) -> Result<(), WavepeekError> {
        let parent_hidden = self.stack.iter().any(|entry| entry.hidden);
        if parent_hidden || record.hidden {
            self.stack.push(StackEntry {
                scope_index: None,
                hidden: true,
            });
            return Ok(());
        }

        let name = normalize_name(record.name.as_str())?;
        let parent = self.stack.iter().rev().find_map(|entry| entry.scope_index);
        let path = match parent {
            Some(parent_idx) => format!("{}.{}", self.scopes[parent_idx].path, name),
            None => name.clone(),
        };
        let depth = parent.map_or(0, |parent_idx| self.scopes[parent_idx].depth + 1);

        let scope_index = if let Some(existing) = self.scope_by_path.get(path.as_str()).copied() {
            existing
        } else {
            let scope_index = self.scopes.len();
            self.scopes.push(ScopeNode {
                name,
                path: path.clone(),
                kind: scope_kind_alias(record.kind).to_string(),
                depth,
                parent,
                children: Vec::new(),
                signals: Vec::new(),
            });
            self.scope_by_path.insert(path, scope_index);
            match parent {
                Some(parent_idx) => push_unique(&mut self.scopes[parent_idx].children, scope_index),
                None => push_unique(&mut self.roots, scope_index),
            }
            scope_index
        };

        self.stack.push(StackEntry {
            scope_index: Some(scope_index),
            hidden: false,
        });
        Ok(())
    }

    pub(super) fn signal(&mut self, record: RawSignalRecord) -> Result<(), WavepeekError> {
        if self.stack.iter().any(|entry| entry.hidden) {
            return Ok(());
        }

        let Some(scope_index) = self.stack.iter().rev().find_map(|entry| entry.scope_index) else {
            return Ok(());
        };

        let (name, width) =
            normalize_signal_name_and_width(record.name.as_str(), record.left, record.right)?;
        let path = format!("{}.{}", self.scopes[scope_index].path, name);
        if self.signal_by_path.contains_key(path.as_str()) {
            return Ok(());
        }

        let kind = self
            .signal_kind_alias(record.kind, record.datatype_id)
            .to_string();
        let signal_index = self.signals.len();
        self.signals.push(FsdbSignalInfo {
            name,
            path: path.clone(),
            kind,
            width,
            idcode: record.idcode,
        });
        self.signal_by_path.insert(path, signal_index);
        self.scopes[scope_index].signals.push(signal_index);
        Ok(())
    }

    pub(super) fn datatype(&mut self, record: RawDatatypeRecord) -> Result<(), WavepeekError> {
        self.datatypes.insert(record.idcode, record.kind);
        Ok(())
    }

    pub(super) fn upscope(&mut self) -> Result<(), WavepeekError> {
        self.stack.pop().ok_or_else(|| {
            WavepeekError::File("FSDB Reader hierarchy emitted upscope without a scope".to_string())
        })?;
        Ok(())
    }

    pub(super) fn end_tree(&mut self) {
        self.stack.clear();
    }

    pub(super) fn finish(mut self) -> FsdbHierarchyIndex {
        let scope_sort_keys = self
            .scopes
            .iter()
            .map(|scope| (scope.name.clone(), scope.path.clone()))
            .collect::<Vec<_>>();
        let signal_sort_keys = self
            .signals
            .iter()
            .map(|signal| (signal.name.clone(), signal.path.clone()))
            .collect::<Vec<_>>();
        for scope in &mut self.scopes {
            sort_indices_by_keys(&scope_sort_keys, &mut scope.children);
            sort_indices_by_keys(&signal_sort_keys, &mut scope.signals);
        }
        sort_indices_by_keys(&scope_sort_keys, &mut self.roots);

        FsdbHierarchyIndex {
            scopes: self.scopes,
            signals: self.signals,
            roots: self.roots,
            scope_by_path: self.scope_by_path,
            signal_by_path: self.signal_by_path,
        }
    }

    fn signal_kind_alias(&self, raw_kind: RawSignalKind, datatype_id: Option<u16>) -> &'static str {
        if let Some(datatype_id) = datatype_id
            && let Some(datatype_kind) = self.datatypes.get(&datatype_id)
            && let Some(alias) = datatype_signal_kind_alias(*datatype_kind)
        {
            return alias;
        }
        signal_kind_alias(raw_kind)
    }
}

impl Default for FsdbHierarchyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FsdbHierarchyIndex {
    pub(super) fn scopes_depth_first(&self, max_depth: Option<usize>) -> Vec<ScopeEntry> {
        let mut entries = Vec::new();
        for root in &self.roots {
            self.collect_scope_entries(*root, max_depth, &mut entries);
        }
        entries
    }

    pub(super) fn signals_in_scope(
        &self,
        scope_path: &str,
    ) -> Result<Vec<SignalEntry>, WavepeekError> {
        let scope_index = self.scope_index(scope_path)?;
        Ok(self.scopes[scope_index]
            .signals
            .iter()
            .map(|signal_index| signal_entry(&self.signals[*signal_index]))
            .collect())
    }

    pub(super) fn signals_in_scope_recursive(
        &self,
        scope_path: &str,
        max_depth: Option<usize>,
    ) -> Result<Vec<SignalEntry>, WavepeekError> {
        let scope_index = self.scope_index(scope_path)?;
        let mut entries = Vec::new();
        self.collect_signal_entries(scope_index, 0, max_depth, &mut entries);
        Ok(entries)
    }

    pub(super) fn resolve_signal(
        &self,
        canonical_path: &str,
    ) -> Result<ResolvedSignal, WavepeekError> {
        let signal = self.signal_info(canonical_path)?;
        Ok(ResolvedSignal {
            path: signal.path.clone(),
            id: SignalId::from_backend_index(signal.idcode),
            width: signal.width.unwrap_or(1),
        })
    }

    pub(super) fn resolve_expr_signal(
        &self,
        canonical_path: &str,
    ) -> Result<ExprResolvedSignal, WavepeekError> {
        let signal = self.signal_info(canonical_path)?;
        Ok(ExprResolvedSignal {
            path: signal.path.clone(),
            id: SignalId::from_backend_index(signal.idcode),
            expr_type: expr_type_from_signal(signal),
        })
    }

    pub(super) fn signal_count(&self) -> usize {
        self.signals.len()
    }

    pub(super) fn scope_count(&self) -> usize {
        self.scopes.len()
    }

    fn collect_scope_entries(
        &self,
        scope_index: usize,
        max_depth: Option<usize>,
        entries: &mut Vec<ScopeEntry>,
    ) {
        let scope = &self.scopes[scope_index];
        if let Some(max_depth) = max_depth
            && scope.depth > max_depth
        {
            return;
        }
        entries.push(ScopeEntry {
            path: scope.path.clone(),
            depth: scope.depth,
            kind: scope.kind.clone(),
        });
        if max_depth == Some(scope.depth) {
            return;
        }
        for child in &scope.children {
            self.collect_scope_entries(*child, max_depth, entries);
        }
    }

    fn collect_signal_entries(
        &self,
        scope_index: usize,
        depth: usize,
        max_depth: Option<usize>,
        entries: &mut Vec<SignalEntry>,
    ) {
        if let Some(max_depth) = max_depth
            && depth > max_depth
        {
            return;
        }

        let scope = &self.scopes[scope_index];
        entries.extend(
            scope
                .signals
                .iter()
                .map(|signal_index| signal_entry(&self.signals[*signal_index])),
        );
        if max_depth == Some(depth) {
            return;
        }
        for child in &scope.children {
            self.collect_signal_entries(*child, depth + 1, max_depth, entries);
        }
    }

    fn scope_index(&self, scope_path: &str) -> Result<usize, WavepeekError> {
        self.scope_by_path
            .get(scope_path)
            .copied()
            .ok_or_else(|| WavepeekError::Scope(format!("scope '{scope_path}' not found in dump")))
    }

    fn signal_info(&self, canonical_path: &str) -> Result<&FsdbSignalInfo, WavepeekError> {
        self.signal_by_path
            .get(canonical_path)
            .map(|index| &self.signals[*index])
            .ok_or_else(|| {
                WavepeekError::Signal(format!("signal '{canonical_path}' not found in dump"))
            })
    }
}

fn signal_entry(signal: &FsdbSignalInfo) -> SignalEntry {
    SignalEntry {
        name: signal.name.clone(),
        path: signal.path.clone(),
        kind: signal.kind.clone(),
        width: signal.width,
    }
}

fn expr_type_from_signal(signal: &FsdbSignalInfo) -> ExprType {
    let width = signal.width.unwrap_or(1);
    let storage = if width > 1 {
        ExprStorage::PackedVector
    } else {
        ExprStorage::Scalar
    };
    match signal.kind.as_str() {
        "real" | "real_time" | "real_parameter" | "short_real" => ExprType {
            kind: ExprTypeKind::Real,
            storage: ExprStorage::Scalar,
            width: 64,
            is_four_state: false,
            is_signed: false,
            enum_type_id: None,
            enum_labels: None,
        },
        "string" => ExprType {
            kind: ExprTypeKind::String,
            storage: ExprStorage::Scalar,
            width: 0,
            is_four_state: false,
            is_signed: false,
            enum_type_id: None,
            enum_labels: None,
        },
        "event" => ExprType {
            kind: ExprTypeKind::Event,
            storage: ExprStorage::Scalar,
            width: 0,
            is_four_state: false,
            is_signed: false,
            enum_type_id: None,
            enum_labels: None,
        },
        "byte" => integer_expr_type(IntegerLikeKind::Byte, 8),
        "short_int" => integer_expr_type(IntegerLikeKind::Shortint, 16),
        "int" => integer_expr_type(IntegerLikeKind::Int, 32),
        "long_int" => integer_expr_type(IntegerLikeKind::Longint, 64),
        "integer" => integer_expr_type(IntegerLikeKind::Integer, 32),
        "time" => integer_expr_type(IntegerLikeKind::Time, 64),
        "enum" => ExprType {
            kind: ExprTypeKind::EnumCore,
            storage,
            width,
            is_four_state: true,
            is_signed: false,
            enum_type_id: None,
            enum_labels: None,
        },
        _ => ExprType {
            kind: ExprTypeKind::BitVector,
            storage,
            width,
            is_four_state: !matches!(signal.kind.as_str(), "bit" | "boolean"),
            is_signed: false,
            enum_type_id: None,
            enum_labels: None,
        },
    }
}

fn integer_expr_type(kind: IntegerLikeKind, width: u32) -> ExprType {
    ExprType {
        kind: ExprTypeKind::IntegerLike(kind),
        storage: ExprStorage::Scalar,
        width,
        is_four_state: matches!(kind, IntegerLikeKind::Integer | IntegerLikeKind::Time),
        is_signed: !matches!(kind, IntegerLikeKind::Time),
        enum_type_id: None,
        enum_labels: None,
    }
}

fn normalize_name(name: &str) -> Result<String, WavepeekError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(WavepeekError::File(
            "FSDB Reader hierarchy emitted an empty name".to_string(),
        ));
    }
    Ok(name.replace('/', "."))
}

fn normalize_signal_name_and_width(
    raw_name: &str,
    left: Option<i32>,
    right: Option<i32>,
) -> Result<(String, Option<u32>), WavepeekError> {
    let raw_name = normalize_name(raw_name)?;
    let width = match (left, right) {
        (Some(left), Some(right)) => Some(bit_width(left, right)?),
        _ => None,
    };
    let name = match (left, right) {
        (Some(left), Some(right)) if left != right => {
            let suffix = format!("[{left}:{right}]");
            raw_name
                .strip_suffix(suffix.as_str())
                .unwrap_or(raw_name.as_str())
                .to_string()
        }
        _ => raw_name,
    };
    Ok((name, width))
}

fn bit_width(left: i32, right: i32) -> Result<u32, WavepeekError> {
    let left = i64::from(left);
    let right = i64::from(right);
    let width = left
        .abs_diff(right)
        .checked_add(1)
        .ok_or_else(|| WavepeekError::File("FSDB signal bit range width overflowed".to_string()))?;
    u32::try_from(width).map_err(|_| {
        WavepeekError::File("FSDB signal bit range exceeds supported width".to_string())
    })
}

fn push_unique(values: &mut Vec<usize>, value: usize) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn sort_indices_by_keys(keys: &[(String, String)], indices: &mut [usize]) {
    indices.sort_by(|lhs, rhs| keys[*lhs].cmp(&keys[*rhs]));
}

fn scope_kind_alias(kind: RawScopeKind) -> &'static str {
    match kind {
        RawScopeKind::Module => "module",
        RawScopeKind::Task => "task",
        RawScopeKind::Function => "function",
        RawScopeKind::Begin => "begin",
        RawScopeKind::Fork => "fork",
        RawScopeKind::Generate => "generate",
        RawScopeKind::Struct => "struct",
        RawScopeKind::Union => "union",
        RawScopeKind::Class => "class",
        RawScopeKind::Interface => "interface",
        RawScopeKind::Package => "package",
        RawScopeKind::Program => "program",
        RawScopeKind::Unknown => "unknown",
    }
}

fn signal_kind_alias(kind: RawSignalKind) -> &'static str {
    match kind {
        RawSignalKind::Event => "event",
        RawSignalKind::Integer => "integer",
        RawSignalKind::Parameter => "parameter",
        RawSignalKind::Real => "real",
        RawSignalKind::Reg => "reg",
        RawSignalKind::Supply0 => "supply0",
        RawSignalKind::Supply1 => "supply1",
        RawSignalKind::Time => "time",
        RawSignalKind::Tri => "tri",
        RawSignalKind::TriAnd => "triand",
        RawSignalKind::TriOr => "trior",
        RawSignalKind::TriReg => "trireg",
        RawSignalKind::Tri0 => "tri0",
        RawSignalKind::Tri1 => "tri1",
        RawSignalKind::WAnd => "wand",
        RawSignalKind::Wire => "wire",
        RawSignalKind::WOr => "wor",
        RawSignalKind::String => "string",
        RawSignalKind::Port => "port",
        RawSignalKind::SparseArray => "sparse_array",
        RawSignalKind::RealTime => "real_time",
        RawSignalKind::RealParameter => "real_parameter",
        RawSignalKind::Bit => "bit",
        RawSignalKind::Logic => "logic",
        RawSignalKind::Int => "int",
        RawSignalKind::ShortInt => "short_int",
        RawSignalKind::LongInt => "long_int",
        RawSignalKind::Byte => "byte",
        RawSignalKind::Enum => "enum",
        RawSignalKind::ShortReal => "short_real",
        RawSignalKind::Boolean => "boolean",
        RawSignalKind::BitVector | RawSignalKind::Unknown => "bit_vector",
    }
}

fn datatype_signal_kind_alias(kind: RawDatatypeKind) -> Option<&'static str> {
    match kind {
        RawDatatypeKind::Enum => Some("enum"),
        RawDatatypeKind::Logic => Some("logic"),
        RawDatatypeKind::Bit => Some("bit"),
        RawDatatypeKind::Int | RawDatatypeKind::UInt => Some("int"),
        RawDatatypeKind::ShortInt | RawDatatypeKind::ShortUInt => Some("short_int"),
        RawDatatypeKind::LongInt | RawDatatypeKind::LongUInt => Some("long_int"),
        RawDatatypeKind::Byte | RawDatatypeKind::UByte => Some("byte"),
        RawDatatypeKind::Real => Some("real"),
        RawDatatypeKind::ShortReal => Some("short_real"),
        RawDatatypeKind::Time => Some("time"),
        RawDatatypeKind::String => Some("string"),
        RawDatatypeKind::Event => Some("event"),
        RawDatatypeKind::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fsdb_hierarchy_sorts_scopes_and_filters_max_depth() {
        let mut builder = FsdbHierarchyBuilder::new();
        builder.begin_tree();
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        builder.scope(scope("z", RawScopeKind::Module)).unwrap();
        builder.upscope().unwrap();
        builder.scope(scope("a", RawScopeKind::Module)).unwrap();
        builder.upscope().unwrap();
        builder.upscope().unwrap();
        builder.scope(scope("alpha", RawScopeKind::Module)).unwrap();
        builder.end_tree();
        let index = builder.finish();

        assert_eq!(
            index.scopes_depth_first(None),
            vec![
                ScopeEntry {
                    path: "alpha".to_string(),
                    depth: 0,
                    kind: "module".to_string()
                },
                ScopeEntry {
                    path: "top".to_string(),
                    depth: 0,
                    kind: "module".to_string()
                },
                ScopeEntry {
                    path: "top.a".to_string(),
                    depth: 1,
                    kind: "module".to_string()
                },
                ScopeEntry {
                    path: "top.z".to_string(),
                    depth: 1,
                    kind: "module".to_string()
                },
            ]
        );
        assert_eq!(
            index.scopes_depth_first(Some(0)),
            vec![
                ScopeEntry {
                    path: "alpha".to_string(),
                    depth: 0,
                    kind: "module".to_string()
                },
                ScopeEntry {
                    path: "top".to_string(),
                    depth: 0,
                    kind: "module".to_string()
                },
            ]
        );
    }

    #[test]
    fn fsdb_hierarchy_excludes_hidden_subtrees() {
        let mut builder = FsdbHierarchyBuilder::new();
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        builder
            .scope(RawScopeRecord {
                name: "secret".to_string(),
                kind: RawScopeKind::Module,
                hidden: true,
            })
            .unwrap();
        builder
            .signal(signal(1, "hidden", RawSignalKind::Wire))
            .unwrap();
        builder.scope(scope("child", RawScopeKind::Module)).unwrap();
        builder.upscope().unwrap();
        builder.upscope().unwrap();
        builder
            .signal(signal(2, "visible", RawSignalKind::Wire))
            .unwrap();
        let index = builder.finish();

        assert_eq!(index.scopes_depth_first(None).len(), 1);
        assert_eq!(
            index.signals_in_scope("top").unwrap(),
            vec![SignalEntry {
                name: "visible".to_string(),
                path: "top.visible".to_string(),
                kind: "wire".to_string(),
                width: None,
            }]
        );
        assert!(index.signals_in_scope("top.secret").is_err());
    }

    #[test]
    fn fsdb_hierarchy_deduplicates_scopes_and_signals() {
        let mut builder = FsdbHierarchyBuilder::new();
        for id in [1, 2] {
            builder.scope(scope("top", RawScopeKind::Module)).unwrap();
            builder
                .signal(signal(id, "clk", RawSignalKind::Wire))
                .unwrap();
            builder.upscope().unwrap();
        }
        let index = builder.finish();

        assert_eq!(index.scopes_depth_first(None).len(), 1);
        assert_eq!(index.signals_in_scope("top").unwrap().len(), 1);
        assert_eq!(
            index.resolve_signal("top.clk").unwrap().id,
            SignalId::from_backend_index(1)
        );
    }

    #[test]
    fn fsdb_hierarchy_normalizes_packed_range_suffixes() {
        let mut builder = FsdbHierarchyBuilder::new();
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        builder
            .signal(signal_with_range(1, "A[3:0]", RawSignalKind::Wire, 3, 0))
            .unwrap();
        builder
            .signal(signal_with_range(
                2,
                "a[0][1][7:0]",
                RawSignalKind::Wire,
                7,
                0,
            ))
            .unwrap();
        builder
            .signal(signal_with_range(3, "B[3]", RawSignalKind::Wire, 3, 3))
            .unwrap();
        let signals = builder.finish().signals_in_scope("top").unwrap();

        assert_eq!(signals[0].name, "A");
        assert_eq!(signals[0].width, Some(4));
        assert_eq!(signals[1].name, "B[3]");
        assert_eq!(signals[1].width, Some(1));
        assert_eq!(signals[2].name, "a[0][1]");
        assert_eq!(signals[2].width, Some(8));
    }

    #[test]
    fn fsdb_hierarchy_lists_direct_and_recursive_signals() {
        let mut builder = FsdbHierarchyBuilder::new();
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        builder
            .signal(signal(1, "clk", RawSignalKind::Wire))
            .unwrap();
        builder.scope(scope("cpu", RawScopeKind::Module)).unwrap();
        builder
            .signal(signal(2, "valid", RawSignalKind::Reg))
            .unwrap();
        builder.upscope().unwrap();
        builder.scope(scope("mem", RawScopeKind::Module)).unwrap();
        builder
            .signal(signal(3, "ready", RawSignalKind::Wire))
            .unwrap();
        let index = builder.finish();

        assert_eq!(
            paths(index.signals_in_scope("top").unwrap()),
            vec!["top.clk".to_string()]
        );
        assert_eq!(
            paths(index.signals_in_scope_recursive("top", None).unwrap()),
            vec![
                "top.clk".to_string(),
                "top.cpu.valid".to_string(),
                "top.mem.ready".to_string(),
            ]
        );
        assert_eq!(
            paths(index.signals_in_scope_recursive("top", Some(0)).unwrap()),
            vec!["top.clk".to_string()]
        );
    }

    #[test]
    fn fsdb_hierarchy_reports_missing_scope_and_signal_errors() {
        let mut builder = FsdbHierarchyBuilder::new();
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        let index = builder.finish();

        assert_eq!(
            index.signals_in_scope("missing").unwrap_err().to_string(),
            "error: scope: scope 'missing' not found in dump"
        );
        assert_eq!(
            index.resolve_signal("top.missing").unwrap_err().to_string(),
            "error: signal: signal 'top.missing' not found in dump"
        );
    }

    #[test]
    fn fsdb_hierarchy_kind_aliases_stay_inside_stable_contract() {
        let scope_aliases = [
            RawScopeKind::Module,
            RawScopeKind::Task,
            RawScopeKind::Function,
            RawScopeKind::Begin,
            RawScopeKind::Fork,
            RawScopeKind::Generate,
            RawScopeKind::Struct,
            RawScopeKind::Union,
            RawScopeKind::Class,
            RawScopeKind::Interface,
            RawScopeKind::Package,
            RawScopeKind::Program,
            RawScopeKind::Unknown,
        ];
        for kind in scope_aliases {
            let alias = scope_kind_alias(kind);
            assert!(STABLE_SCOPE_KIND_ALIASES.contains(&alias));
            assert!(!EXCLUDED_SCOPE_KIND_ALIASES.contains(&alias));
        }

        let signal_aliases = [
            RawSignalKind::Event,
            RawSignalKind::Integer,
            RawSignalKind::Parameter,
            RawSignalKind::Real,
            RawSignalKind::Reg,
            RawSignalKind::Supply0,
            RawSignalKind::Supply1,
            RawSignalKind::Time,
            RawSignalKind::Tri,
            RawSignalKind::TriAnd,
            RawSignalKind::TriOr,
            RawSignalKind::TriReg,
            RawSignalKind::Tri0,
            RawSignalKind::Tri1,
            RawSignalKind::WAnd,
            RawSignalKind::Wire,
            RawSignalKind::WOr,
            RawSignalKind::String,
            RawSignalKind::Port,
            RawSignalKind::SparseArray,
            RawSignalKind::RealTime,
            RawSignalKind::RealParameter,
            RawSignalKind::Bit,
            RawSignalKind::Logic,
            RawSignalKind::Int,
            RawSignalKind::ShortInt,
            RawSignalKind::LongInt,
            RawSignalKind::Byte,
            RawSignalKind::Enum,
            RawSignalKind::ShortReal,
            RawSignalKind::Boolean,
            RawSignalKind::BitVector,
            RawSignalKind::Unknown,
        ];
        for kind in signal_aliases {
            let alias = signal_kind_alias(kind);
            assert!(STABLE_SIGNAL_KIND_ALIASES.contains(&alias));
            assert!(!EXCLUDED_SIGNAL_KIND_ALIASES.contains(&alias));
        }
    }

    #[test]
    fn fsdb_hierarchy_datatype_enum_overrides_signal_kind() {
        let mut builder = FsdbHierarchyBuilder::new();
        builder
            .datatype(RawDatatypeRecord {
                idcode: 7,
                kind: RawDatatypeKind::Enum,
            })
            .unwrap();
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        let mut raw = signal_with_range(1, "state[1:0]", RawSignalKind::Logic, 1, 0);
        raw.datatype_id = Some(7);
        builder.signal(raw).unwrap();
        let index = builder.finish();
        let entry = index.signals_in_scope("top").unwrap().pop().unwrap();

        assert_eq!(entry.kind, "enum");
        assert_eq!(entry.name, "state");
        assert_eq!(entry.width, Some(2));
    }

    fn scope(name: &str, kind: RawScopeKind) -> RawScopeRecord {
        RawScopeRecord {
            name: name.to_string(),
            kind,
            hidden: false,
        }
    }

    fn signal(idcode: u64, name: &str, kind: RawSignalKind) -> RawSignalRecord {
        RawSignalRecord {
            idcode,
            name: name.to_string(),
            kind,
            left: None,
            right: None,
            datatype_id: None,
        }
    }

    fn signal_with_range(
        idcode: u64,
        name: &str,
        kind: RawSignalKind,
        left: i32,
        right: i32,
    ) -> RawSignalRecord {
        RawSignalRecord {
            idcode,
            name: name.to_string(),
            kind,
            left: Some(left),
            right: Some(right),
            datatype_id: None,
        }
    }

    fn paths(entries: Vec<SignalEntry>) -> Vec<String> {
        entries.into_iter().map(|entry| entry.path).collect()
    }
}
