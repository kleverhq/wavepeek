#![allow(dead_code)]

use std::collections::HashMap;

use crate::error::WavepeekError;
use crate::expr::{EnumLabelInfo, ExprStorage, ExprType, ExprTypeKind, IntegerLikeKind};

use super::types::{ExprResolvedSignal, ResolvedSignal, ScopeEntry, SignalEntry, SignalId};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FsdbValueEncoding {
    BitVector,
    Unsupported,
    DatatypeCandidate,
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
    pub(super) datatype_id: Option<u32>,
    pub(super) value_encoding: FsdbValueEncoding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawDatatypeRecord {
    pub(super) idcode: u32,
    pub(super) kind: RawDatatypeKind,
    pub(super) type_name: Option<String>,
    pub(super) bit_width: Option<u32>,
    pub(super) is_signed: Option<bool>,
    pub(super) enum_labels: Option<Vec<EnumLabelInfo>>,
}

#[derive(Debug)]
pub(super) struct FsdbHierarchyBuilder {
    scopes: Vec<ScopeNode>,
    scope_by_path: HashMap<String, usize>,
    scope_origins: HashMap<String, ScopePathOrigin>,
    signal_by_path: HashMap<String, usize>,
    signals: Vec<FsdbSignalInfo>,
    roots: Vec<usize>,
    stack: Vec<StackEntry>,
    current_tree_generation: usize,
    datatypes: HashMap<u32, RawDatatypeRecord>,
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
    value_encoding: FsdbValueEncoding,
    datatype: Option<RawDatatypeRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScopePathOrigin {
    raw_components: Vec<String>,
    last_tree_generation: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StackEntry {
    scope_index: Option<usize>,
    hidden: bool,
    raw_components: Option<Vec<String>>,
}

impl FsdbHierarchyBuilder {
    pub(super) fn new() -> Self {
        Self {
            scopes: Vec::new(),
            scope_by_path: HashMap::new(),
            scope_origins: HashMap::new(),
            signal_by_path: HashMap::new(),
            signals: Vec::new(),
            roots: Vec::new(),
            stack: Vec::new(),
            current_tree_generation: 0,
            datatypes: HashMap::new(),
        }
    }

    pub(super) fn begin_tree(&mut self) {
        self.stack.clear();
        self.current_tree_generation = self.current_tree_generation.saturating_add(1);
    }

    pub(super) fn scope(&mut self, record: RawScopeRecord) -> Result<(), WavepeekError> {
        let parent_hidden = self.stack.iter().any(|entry| entry.hidden);
        if parent_hidden || record.hidden {
            self.stack.push(StackEntry {
                scope_index: None,
                hidden: true,
                raw_components: None,
            });
            return Ok(());
        }

        let raw_local_name = raw_hierarchy_name(record.name.as_str())?;
        let raw_name = normalize_name(record.name.as_str())?;
        let (name, _) = normalize_vcd_escaped_identifier(raw_name.as_str());
        let parent_entry = self
            .stack
            .iter()
            .rev()
            .find(|entry| entry.scope_index.is_some());
        let parent = parent_entry.and_then(|entry| entry.scope_index);
        let mut raw_components = parent_entry
            .and_then(|entry| entry.raw_components.clone())
            .unwrap_or_default();
        raw_components.push(raw_local_name);
        let path = match parent {
            Some(parent_idx) => format!("{}.{}", self.scopes[parent_idx].path, name),
            None => name.clone(),
        };
        let kind = scope_kind_alias(record.kind).to_string();
        let depth = parent.map_or(0, |parent_idx| self.scopes[parent_idx].depth + 1);

        let scope_index = if let Some(existing) = self.scope_by_path.get(path.as_str()).copied() {
            self.merge_existing_scope_path(path.as_str(), raw_components.as_slice())?;
            if self.scopes[existing].kind != kind {
                return Err(ambiguous_scope_path_error(path.as_str()));
            }
            existing
        } else {
            let scope_index = self.scopes.len();
            self.scopes.push(ScopeNode {
                name,
                path: path.clone(),
                kind,
                depth,
                parent,
                children: Vec::new(),
                signals: Vec::new(),
            });
            self.scope_by_path.insert(path.clone(), scope_index);
            self.scope_origins.insert(
                path,
                ScopePathOrigin {
                    raw_components: raw_components.clone(),
                    last_tree_generation: self.current_tree_generation,
                },
            );
            match parent {
                Some(parent_idx) => push_unique(&mut self.scopes[parent_idx].children, scope_index),
                None => push_unique(&mut self.roots, scope_index),
            }
            scope_index
        };

        self.stack.push(StackEntry {
            scope_index: Some(scope_index),
            hidden: false,
            raw_components: Some(raw_components),
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

        let (name, range_width) =
            normalize_signal_name_and_width(record.name.as_str(), record.left, record.right)?;
        let (scope_index, name) = self.scope_for_signal_name(scope_index, name)?;
        let path = format!("{}.{}", self.scopes[scope_index].path, name);

        let datatype = record
            .datatype_id
            .and_then(|datatype_id| self.datatypes.get(&datatype_id).cloned());
        let datatype_kind = datatype.as_ref().map(|datatype| datatype.kind);
        let kind = self
            .signal_kind_alias(record.kind, datatype_kind)
            .to_string();
        let width =
            range_width.or_else(|| datatype.as_ref().and_then(|datatype| datatype.bit_width));
        let value_encoding = self.signal_value_encoding(record.value_encoding, datatype_kind);
        let candidate = FsdbSignalInfo {
            name,
            path: path.clone(),
            kind,
            width,
            idcode: record.idcode,
            value_encoding,
            datatype,
        };
        if let Some(existing) = self.signal_by_path.get(path.as_str()).copied() {
            if self.signals[existing] == candidate {
                return Ok(());
            }
            return Err(ambiguous_signal_path_error(path.as_str()));
        }

        let signal_index = self.signals.len();
        self.signals.push(candidate);
        self.signal_by_path.insert(path, signal_index);
        self.scopes[scope_index].signals.push(signal_index);
        Ok(())
    }

    pub(super) fn datatype(&mut self, record: RawDatatypeRecord) -> Result<(), WavepeekError> {
        self.datatypes.insert(record.idcode, record);
        Ok(())
    }

    fn scope_for_signal_name(
        &mut self,
        scope_index: usize,
        name: String,
    ) -> Result<(usize, String), WavepeekError> {
        let mut parts = name.rsplitn(2, '.');
        let signal_name = parts.next().unwrap_or(name.as_str());
        let Some(scope_prefix) = parts.next() else {
            return Ok((scope_index, name));
        };
        if scope_prefix.is_empty() || signal_name.is_empty() {
            return Ok((scope_index, name));
        }

        let mut parent = scope_index;
        for part in scope_prefix.split('.') {
            if part.is_empty() {
                return Ok((scope_index, name));
            }
            parent = self.synthetic_scope(parent, part)?;
        }
        Ok((parent, signal_name.to_string()))
    }

    fn synthetic_scope(&mut self, parent: usize, name: &str) -> Result<usize, WavepeekError> {
        let path = format!("{}.{}", self.scopes[parent].path, name);
        let mut raw_components = self
            .scope_origins
            .get(self.scopes[parent].path.as_str())
            .map(|origin| origin.raw_components.clone())
            .unwrap_or_else(|| vec![self.scopes[parent].name.clone()]);
        raw_components.push(name.to_string());
        if let Some(existing) = self.scope_by_path.get(path.as_str()).copied() {
            self.validate_synthetic_scope_path(
                path.as_str(),
                existing,
                parent,
                raw_components.as_slice(),
            )?;
            return Ok(existing);
        }

        let scope_index = self.scopes.len();
        self.scopes.push(ScopeNode {
            name: name.to_string(),
            path: path.clone(),
            kind: "unknown".to_string(),
            depth: self.scopes[parent].depth + 1,
            parent: Some(parent),
            children: Vec::new(),
            signals: Vec::new(),
        });
        self.scope_by_path.insert(path.clone(), scope_index);
        self.scope_origins.insert(
            path,
            ScopePathOrigin {
                raw_components,
                last_tree_generation: self.current_tree_generation,
            },
        );
        push_unique(&mut self.scopes[parent].children, scope_index);
        Ok(scope_index)
    }

    fn validate_synthetic_scope_path(
        &self,
        path: &str,
        existing: usize,
        parent: usize,
        raw_components: &[String],
    ) -> Result<(), WavepeekError> {
        let origin = self.scope_origins.get(path).ok_or_else(|| {
            WavepeekError::Internal(format!(
                "FSDB hierarchy scope path '{path}' is missing origin metadata"
            ))
        })?;
        if self.scopes[existing].parent != Some(parent) || origin.raw_components != raw_components {
            return Err(ambiguous_scope_path_error(path));
        }
        Ok(())
    }

    fn merge_existing_scope_path(
        &mut self,
        path: &str,
        raw_components: &[String],
    ) -> Result<(), WavepeekError> {
        let origin = self.scope_origins.get_mut(path).ok_or_else(|| {
            WavepeekError::Internal(format!(
                "FSDB hierarchy scope path '{path}' is missing origin metadata"
            ))
        })?;
        if origin.raw_components != raw_components
            || origin.last_tree_generation == self.current_tree_generation
        {
            return Err(ambiguous_scope_path_error(path));
        }
        origin.last_tree_generation = self.current_tree_generation;
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

    fn signal_kind_alias(
        &self,
        raw_kind: RawSignalKind,
        datatype_kind: Option<RawDatatypeKind>,
    ) -> &'static str {
        if let Some(datatype_kind) = datatype_kind
            && let Some(alias) = datatype_signal_kind_alias(datatype_kind)
        {
            return alias;
        }
        signal_kind_alias(raw_kind)
    }

    fn signal_value_encoding(
        &self,
        raw_encoding: FsdbValueEncoding,
        datatype_kind: Option<RawDatatypeKind>,
    ) -> FsdbValueEncoding {
        if let Some(datatype_kind) = datatype_kind {
            if datatype_forces_unsupported_value(datatype_kind) {
                return FsdbValueEncoding::Unsupported;
            }
            if raw_encoding == FsdbValueEncoding::DatatypeCandidate
                && datatype_supports_bit_vector_value(datatype_kind)
            {
                return FsdbValueEncoding::BitVector;
            }
        }
        if raw_encoding == FsdbValueEncoding::DatatypeCandidate {
            FsdbValueEncoding::Unsupported
        } else {
            raw_encoding
        }
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

    pub(super) fn signal_value_encoding(
        &self,
        canonical_path: &str,
    ) -> Result<FsdbValueEncoding, WavepeekError> {
        Ok(self.signal_info(canonical_path)?.value_encoding)
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
    let datatype = signal.datatype.as_ref();
    let datatype_kind = datatype.map(|datatype| datatype.kind);
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
        "byte" => integer_expr_type(
            IntegerLikeKind::Byte,
            8,
            datatype_signedness(datatype)
                .unwrap_or(!matches!(datatype_kind, Some(RawDatatypeKind::UByte))),
        ),
        "short_int" => integer_expr_type(
            IntegerLikeKind::Shortint,
            16,
            datatype_signedness(datatype)
                .unwrap_or(!matches!(datatype_kind, Some(RawDatatypeKind::ShortUInt))),
        ),
        "int" => integer_expr_type(
            IntegerLikeKind::Int,
            32,
            datatype_signedness(datatype)
                .unwrap_or(!matches!(datatype_kind, Some(RawDatatypeKind::UInt))),
        ),
        "long_int" => integer_expr_type(
            IntegerLikeKind::Longint,
            64,
            datatype_signedness(datatype)
                .unwrap_or(!matches!(datatype_kind, Some(RawDatatypeKind::LongUInt))),
        ),
        "integer" => integer_expr_type(IntegerLikeKind::Integer, 32, true),
        "time" => integer_expr_type(IntegerLikeKind::Time, 64, false),
        "enum" => enum_expr_type(datatype, storage, width),
        _ => ExprType {
            kind: ExprTypeKind::BitVector,
            storage,
            width,
            is_four_state: !matches!(signal.kind.as_str(), "bit" | "boolean"),
            is_signed: datatype_signedness(datatype).unwrap_or(false),
            enum_type_id: None,
            enum_labels: None,
        },
    }
}

fn integer_expr_type(kind: IntegerLikeKind, width: u32, is_signed: bool) -> ExprType {
    ExprType {
        kind: ExprTypeKind::IntegerLike(kind),
        storage: ExprStorage::Scalar,
        width,
        is_four_state: matches!(kind, IntegerLikeKind::Integer | IntegerLikeKind::Time),
        is_signed,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn enum_expr_type(
    datatype: Option<&RawDatatypeRecord>,
    storage: ExprStorage,
    width: u32,
) -> ExprType {
    let is_signed = datatype_signedness(datatype).unwrap_or(false);
    let enum_labels = datatype
        .and_then(|datatype| datatype.enum_labels.clone())
        .map(|labels| normalize_enum_labels_for_width(labels, width, is_signed))
        .filter(|labels| !labels.is_empty());
    let enum_type_id = datatype.and_then(|datatype| {
        datatype
            .type_name
            .as_ref()
            .filter(|name| !name.trim().is_empty())
            .cloned()
            .or_else(|| {
                enum_labels
                    .as_ref()
                    .map(|_| format!("fsdb-dt:{}", datatype.idcode))
            })
    });
    ExprType {
        kind: ExprTypeKind::EnumCore,
        storage,
        width,
        is_four_state: true,
        is_signed,
        enum_type_id,
        enum_labels,
    }
}

fn normalize_enum_labels_for_width(
    labels: Vec<EnumLabelInfo>,
    width: u32,
    is_signed: bool,
) -> Vec<EnumLabelInfo> {
    labels
        .into_iter()
        .map(|label| EnumLabelInfo {
            name: label.name,
            bits: resize_enum_label_bits(label.bits.as_str(), width, is_signed),
        })
        .collect()
}

fn resize_enum_label_bits(bits: &str, width: u32, is_signed: bool) -> String {
    let target_width = width as usize;
    if bits.is_empty() || bits.len() == target_width {
        return bits.to_string();
    }
    if bits.len() > target_width {
        return bits[bits.len() - target_width..].to_string();
    }

    let extension = if is_signed {
        bits.as_bytes()[0] as char
    } else {
        '0'
    };
    let mut resized = String::with_capacity(target_width);
    resized.extend(std::iter::repeat_n(extension, target_width - bits.len()));
    resized.push_str(bits);
    resized
}

fn datatype_signedness(datatype: Option<&RawDatatypeRecord>) -> Option<bool> {
    let datatype = datatype?;
    datatype.is_signed.or(match datatype.kind {
        RawDatatypeKind::Int
        | RawDatatypeKind::ShortInt
        | RawDatatypeKind::LongInt
        | RawDatatypeKind::Byte => Some(true),
        RawDatatypeKind::UInt
        | RawDatatypeKind::ShortUInt
        | RawDatatypeKind::LongUInt
        | RawDatatypeKind::UByte
        | RawDatatypeKind::Time => Some(false),
        RawDatatypeKind::Enum
        | RawDatatypeKind::Logic
        | RawDatatypeKind::Bit
        | RawDatatypeKind::Real
        | RawDatatypeKind::ShortReal
        | RawDatatypeKind::String
        | RawDatatypeKind::Event
        | RawDatatypeKind::Unknown => None,
    })
}

fn raw_hierarchy_name(name: &str) -> Result<String, WavepeekError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(WavepeekError::File(
            "FSDB Reader hierarchy emitted an empty name".to_string(),
        ));
    }
    Ok(name.to_string())
}

fn normalize_name(name: &str) -> Result<String, WavepeekError> {
    Ok(raw_hierarchy_name(name)?.replace('/', "."))
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
        (Some(left), Some(right)) => {
            let suffix = format!("[{left}:{right}]");
            raw_name
                .strip_suffix(suffix.as_str())
                .unwrap_or(raw_name.as_str())
                .to_string()
        }
        _ => raw_name,
    };
    let (mut name, was_escaped) = normalize_vcd_escaped_identifier(name.as_str());
    if was_escaped {
        if width == Some(1) {
            name = strip_scalar_bit_select_suffix(name.as_str()).unwrap_or(name);
        } else if let (Some(left), Some(right)) = (left, right)
            && left == right
        {
            name = normalize_scalar_bit_select_name(name.as_str(), left).unwrap_or(name);
        } else {
            name = normalize_scalar_bit_select_name_from_suffix(name.as_str()).unwrap_or(name);
        }
    } else {
        name = strip_scalar_bit_select_suffix(name.as_str()).unwrap_or(name);
    }
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(WavepeekError::File(
            "FSDB Reader hierarchy emitted an empty signal name".to_string(),
        ));
    }
    Ok((name, width))
}

fn normalize_vcd_escaped_identifier(name: &str) -> (String, bool) {
    let Some(rest) = name.strip_prefix('\\') else {
        return (name.to_string(), false);
    };
    (rest.trim_end().to_string(), true)
}

fn normalize_scalar_bit_select_name(name: &str, bit: i32) -> Option<String> {
    let suffix = format!("[{bit}]");
    scalar_bit_select_base(name, suffix.as_str()).map(|base| format!("{base}.{suffix}"))
}

fn normalize_scalar_bit_select_name_from_suffix(name: &str) -> Option<String> {
    let suffix_start = name.rfind('[')?;
    let suffix = name.get(suffix_start..)?;
    if !suffix.ends_with(']') {
        return None;
    }
    let digits = suffix.strip_prefix('[')?.strip_suffix(']')?;
    if digits.is_empty() || !digits.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    scalar_bit_select_base(name, suffix).map(|base| format!("{base}.{suffix}"))
}

fn strip_scalar_bit_select_suffix(name: &str) -> Option<String> {
    let suffix_start = name.rfind('[')?;
    let suffix = name.get(suffix_start..)?;
    if !suffix.ends_with(']') {
        return None;
    }
    let digits = suffix.strip_prefix('[')?.strip_suffix(']')?;
    if digits.is_empty() || !digits.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    scalar_bit_select_base(name, suffix).map(str::to_string)
}

fn scalar_bit_select_base<'a>(name: &'a str, suffix: &str) -> Option<&'a str> {
    let base = name.strip_suffix(suffix)?;
    if base.is_empty() || base.ends_with('.') || base.contains('[') || base.contains(']') {
        return None;
    }
    Some(base)
}

fn ambiguous_scope_path_error(path: &str) -> WavepeekError {
    WavepeekError::File(format!(
        "FSDB hierarchy contains ambiguous canonical scope path '{path}'"
    ))
}

fn ambiguous_signal_path_error(path: &str) -> WavepeekError {
    WavepeekError::File(format!(
        "FSDB hierarchy contains ambiguous canonical signal path '{path}'"
    ))
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

fn datatype_forces_unsupported_value(kind: RawDatatypeKind) -> bool {
    matches!(
        kind,
        RawDatatypeKind::Real
            | RawDatatypeKind::ShortReal
            | RawDatatypeKind::String
            | RawDatatypeKind::Event
    )
}

fn datatype_supports_bit_vector_value(kind: RawDatatypeKind) -> bool {
    matches!(
        kind,
        RawDatatypeKind::Enum
            | RawDatatypeKind::Logic
            | RawDatatypeKind::Bit
            | RawDatatypeKind::Int
            | RawDatatypeKind::UInt
            | RawDatatypeKind::ShortInt
            | RawDatatypeKind::ShortUInt
            | RawDatatypeKind::LongInt
            | RawDatatypeKind::LongUInt
            | RawDatatypeKind::Byte
            | RawDatatypeKind::UByte
            | RawDatatypeKind::Time
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::waveform::{
        EXCLUDED_SCOPE_KIND_ALIASES, EXCLUDED_SIGNAL_KIND_ALIASES, STABLE_SCOPE_KIND_ALIASES,
        STABLE_SIGNAL_KIND_ALIASES,
    };

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
    fn fsdb_hierarchy_rejects_duplicate_scope_paths_in_one_tree() {
        let mut builder = FsdbHierarchyBuilder::new();
        builder.begin_tree();
        builder.scope(scope("top/a", RawScopeKind::Module)).unwrap();
        builder.upscope().unwrap();

        let error = builder
            .scope(scope("top.a", RawScopeKind::Interface))
            .expect_err("canonical scope collisions should be rejected")
            .to_string();

        assert_eq!(
            error,
            "error: file: FSDB hierarchy contains ambiguous canonical scope path 'top.a'"
        );
    }

    #[test]
    fn fsdb_hierarchy_merges_matching_scope_paths_across_tree_passes() {
        let mut builder = FsdbHierarchyBuilder::new();
        builder.begin_tree();
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        builder
            .signal(signal(1, "clk", RawSignalKind::Wire))
            .unwrap();
        builder.upscope().unwrap();
        builder.end_tree();

        builder.begin_tree();
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        builder
            .signal(signal(2, "ev", RawSignalKind::Event))
            .unwrap();
        builder.upscope().unwrap();
        builder.end_tree();
        let index = builder.finish();

        assert_eq!(index.scopes_depth_first(None).len(), 1);
        assert_eq!(
            paths(index.signals_in_scope("top").unwrap()),
            vec!["top.clk".to_string(), "top.ev".to_string()]
        );
        assert_eq!(
            index.resolve_signal("top.clk").unwrap().id,
            SignalId::from_backend_index(1)
        );
        assert_eq!(
            index.resolve_signal("top.ev").unwrap().id,
            SignalId::from_backend_index(2)
        );
    }

    #[test]
    fn fsdb_hierarchy_rejects_duplicate_signal_paths() {
        let mut builder = FsdbHierarchyBuilder::new();
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        builder
            .signal(signal(1, "clk", RawSignalKind::Wire))
            .unwrap();

        let error = builder
            .signal(signal(2, "\\clk ", RawSignalKind::Reg))
            .expect_err("canonical signal collisions should be rejected")
            .to_string();

        assert_eq!(
            error,
            "error: file: FSDB hierarchy contains ambiguous canonical signal path 'top.clk'"
        );
    }

    #[test]
    fn fsdb_hierarchy_rejects_synthetic_scope_origin_collisions() {
        let mut builder = FsdbHierarchyBuilder::new();
        builder.begin_tree();
        builder.scope(scope("top/a", RawScopeKind::Module)).unwrap();
        builder.upscope().unwrap();
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();

        let error = builder
            .signal(signal(1, "a.sig", RawSignalKind::Wire))
            .expect_err("synthetic scopes should not reuse unrelated canonical paths")
            .to_string();

        assert_eq!(
            error,
            "error: file: FSDB hierarchy contains ambiguous canonical scope path 'top.a'"
        );
    }

    #[test]
    fn fsdb_hierarchy_skips_exact_duplicate_signal_callbacks() {
        let mut builder = FsdbHierarchyBuilder::new();
        let raw = signal(1, "clk", RawSignalKind::Wire);
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        builder.signal(raw.clone()).unwrap();
        builder.signal(raw).unwrap();
        let index = builder.finish();

        assert_eq!(
            index.signals_in_scope("top").unwrap(),
            vec![SignalEntry {
                name: "clk".to_string(),
                path: "top.clk".to_string(),
                kind: "wire".to_string(),
                width: None,
            }]
        );
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
        builder
            .signal(signal_with_range(
                4,
                "\\araddr[0] ",
                RawSignalKind::Wire,
                31,
                0,
            ))
            .unwrap();
        builder
            .signal(signal_with_range(
                5,
                "\\q_err[0] ",
                RawSignalKind::Wire,
                0,
                0,
            ))
            .unwrap();
        builder
            .signal(signal(6, "\\ready ", RawSignalKind::Wire))
            .unwrap();
        let index = builder.finish();
        let signals = index.signals_in_scope("top").unwrap();

        assert_eq!(signals[0].name, "A");
        assert_eq!(signals[0].width, Some(4));
        assert_eq!(signals[1].name, "B");
        assert_eq!(signals[1].width, Some(1));
        assert_eq!(signals[2].name, "a[0][1]");
        assert_eq!(signals[2].width, Some(8));
        assert_eq!(signals[3].name, "q_err");
        assert_eq!(signals[3].width, Some(1));
        assert_eq!(signals[4].name, "ready");
        assert_eq!(signals[4].width, None);

        let escaped_array_element = index.resolve_signal("top.araddr.[0]").unwrap();
        assert_eq!(escaped_array_element.width, 32);
        assert_eq!(escaped_array_element.id, SignalId::from_backend_index(4));
        assert_eq!(index.scope_index("top.araddr").unwrap(), 1);
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
            .datatype(raw_datatype(7, RawDatatypeKind::Enum))
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
        assert_eq!(
            index.signal_value_encoding("top.state").unwrap(),
            FsdbValueEncoding::BitVector
        );
    }

    #[test]
    fn fsdb_hierarchy_datatype_signedness_drives_expression_types() {
        let mut builder = FsdbHierarchyBuilder::new();
        for (idcode, kind) in [
            (31, RawDatatypeKind::Byte),
            (32, RawDatatypeKind::UByte),
            (33, RawDatatypeKind::ShortInt),
            (34, RawDatatypeKind::ShortUInt),
            (35, RawDatatypeKind::Int),
            (36, RawDatatypeKind::UInt),
            (37, RawDatatypeKind::LongInt),
            (38, RawDatatypeKind::LongUInt),
        ] {
            builder.datatype(raw_datatype(idcode, kind)).unwrap();
        }
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        for (idcode, datatype_id, name, left, right) in [
            (1, 31, "sbyte[7:0]", 7, 0),
            (2, 32, "ubyte[7:0]", 7, 0),
            (3, 33, "sshort[15:0]", 15, 0),
            (4, 34, "ushort[15:0]", 15, 0),
            (5, 35, "sint[31:0]", 31, 0),
            (6, 36, "uint[31:0]", 31, 0),
            (7, 37, "slong[63:0]", 63, 0),
            (8, 38, "ulong[63:0]", 63, 0),
        ] {
            let mut raw = signal_with_range(idcode, name, RawSignalKind::Unknown, left, right);
            raw.datatype_id = Some(datatype_id);
            builder.signal(raw).unwrap();
        }
        let index = builder.finish();

        for (path, kind, is_signed, public_kind) in [
            ("top.sbyte", IntegerLikeKind::Byte, true, "byte"),
            ("top.ubyte", IntegerLikeKind::Byte, false, "byte"),
            ("top.sshort", IntegerLikeKind::Shortint, true, "short_int"),
            ("top.ushort", IntegerLikeKind::Shortint, false, "short_int"),
            ("top.sint", IntegerLikeKind::Int, true, "int"),
            ("top.uint", IntegerLikeKind::Int, false, "int"),
            ("top.slong", IntegerLikeKind::Longint, true, "long_int"),
            ("top.ulong", IntegerLikeKind::Longint, false, "long_int"),
        ] {
            let resolved = index.resolve_expr_signal(path).unwrap();
            assert_eq!(
                resolved.expr_type.kind,
                ExprTypeKind::IntegerLike(kind),
                "{path}"
            );
            assert_eq!(resolved.expr_type.is_signed, is_signed, "{path}");
            assert!(!resolved.expr_type.is_four_state, "{path}");
            assert_eq!(
                index
                    .signals_in_scope("top")
                    .unwrap()
                    .into_iter()
                    .find(|signal| signal.path == path)
                    .unwrap()
                    .kind,
                public_kind,
                "{path}"
            );
        }
    }

    #[test]
    fn fsdb_hierarchy_datatype_enum_metadata_drives_expression_type() {
        let mut builder = FsdbHierarchyBuilder::new();
        builder
            .datatype(RawDatatypeRecord {
                idcode: 41,
                kind: RawDatatypeKind::Enum,
                type_name: Some("pkg::state_t".to_string()),
                bit_width: Some(2),
                is_signed: Some(false),
                enum_labels: Some(vec![
                    EnumLabelInfo {
                        name: "IDLE".to_string(),
                        bits: "00".to_string(),
                    },
                    EnumLabelInfo {
                        name: "BUSY".to_string(),
                        bits: "01".to_string(),
                    },
                ]),
            })
            .unwrap();
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        let mut raw = signal(1, "state", RawSignalKind::Logic);
        raw.datatype_id = Some(41);
        builder.signal(raw).unwrap();
        let index = builder.finish();

        let entry = index.signals_in_scope("top").unwrap().pop().unwrap();
        assert_eq!(entry.kind, "enum");
        assert_eq!(entry.width, Some(2));

        let resolved = index.resolve_expr_signal("top.state").unwrap();
        assert_eq!(resolved.expr_type.kind, ExprTypeKind::EnumCore);
        assert_eq!(resolved.expr_type.width, 2);
        assert_eq!(
            resolved.expr_type.enum_type_id.as_deref(),
            Some("pkg::state_t")
        );
        assert_eq!(
            resolved.expr_type.enum_labels,
            Some(vec![
                EnumLabelInfo {
                    name: "IDLE".to_string(),
                    bits: "00".to_string(),
                },
                EnumLabelInfo {
                    name: "BUSY".to_string(),
                    bits: "01".to_string(),
                },
            ])
        );
    }

    #[test]
    fn fsdb_hierarchy_datatype_enum_labels_follow_signal_width() {
        let mut builder = FsdbHierarchyBuilder::new();
        builder
            .datatype(RawDatatypeRecord {
                idcode: 44,
                kind: RawDatatypeKind::Enum,
                type_name: Some("pkg::narrow_state_t".to_string()),
                bit_width: Some(4),
                is_signed: Some(false),
                enum_labels: Some(vec![
                    EnumLabelInfo {
                        name: "ONE".to_string(),
                        bits: "0001".to_string(),
                    },
                    EnumLabelInfo {
                        name: "THREE".to_string(),
                        bits: "0011".to_string(),
                    },
                ]),
            })
            .unwrap();
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        let mut raw = signal_with_range(1, "state[1:0]", RawSignalKind::Logic, 1, 0);
        raw.datatype_id = Some(44);
        builder.signal(raw).unwrap();
        let index = builder.finish();

        let resolved = index.resolve_expr_signal("top.state").unwrap();
        assert_eq!(resolved.expr_type.width, 2);
        assert_eq!(
            resolved.expr_type.enum_labels,
            Some(vec![
                EnumLabelInfo {
                    name: "ONE".to_string(),
                    bits: "01".to_string(),
                },
                EnumLabelInfo {
                    name: "THREE".to_string(),
                    bits: "11".to_string(),
                },
            ])
        );
    }

    #[test]
    fn fsdb_hierarchy_datatype_signedness_drives_packed_vectors() {
        let mut builder = FsdbHierarchyBuilder::new();
        builder
            .datatype(RawDatatypeRecord {
                idcode: 42,
                kind: RawDatatypeKind::Logic,
                type_name: Some("signed_logic_t".to_string()),
                bit_width: Some(8),
                is_signed: Some(true),
                enum_labels: None,
            })
            .unwrap();
        builder
            .datatype(RawDatatypeRecord {
                idcode: 43,
                kind: RawDatatypeKind::Bit,
                type_name: Some("unsigned_bit_t".to_string()),
                bit_width: Some(8),
                is_signed: Some(false),
                enum_labels: None,
            })
            .unwrap();
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        for (idcode, datatype_id, name, kind) in [
            (1, 42, "signed_data", RawSignalKind::Logic),
            (2, 43, "unsigned_data", RawSignalKind::Bit),
        ] {
            let mut raw = signal(idcode, name, kind);
            raw.datatype_id = Some(datatype_id);
            builder.signal(raw).unwrap();
        }
        let index = builder.finish();

        for (path, is_signed) in [("top.signed_data", true), ("top.unsigned_data", false)] {
            let resolved = index.resolve_expr_signal(path).unwrap();
            assert_eq!(resolved.expr_type.kind, ExprTypeKind::BitVector, "{path}");
            assert_eq!(resolved.expr_type.width, 8, "{path}");
            assert_eq!(resolved.expr_type.is_signed, is_signed, "{path}");
        }
    }

    #[test]
    fn fsdb_hierarchy_datatype_candidates_upgrade_for_vector_datatypes() {
        let mut builder = FsdbHierarchyBuilder::new();
        for (idcode, kind) in [
            (21, RawDatatypeKind::Enum),
            (22, RawDatatypeKind::Logic),
            (23, RawDatatypeKind::Int),
            (24, RawDatatypeKind::Unknown),
        ] {
            builder.datatype(raw_datatype(idcode, kind)).unwrap();
        }
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        for (idcode, datatype_id, name) in [
            (1, Some(21), "state[3:0]"),
            (2, Some(22), "bits[3:0]"),
            (3, Some(23), "count[3:0]"),
            (4, Some(24), "mystery[3:0]"),
            (5, None, "untyped[3:0]"),
        ] {
            let mut raw = signal_with_range(idcode, name, RawSignalKind::Unknown, 3, 0);
            raw.datatype_id = datatype_id;
            raw.value_encoding = FsdbValueEncoding::DatatypeCandidate;
            builder.signal(raw).unwrap();
        }
        let index = builder.finish();

        assert_eq!(
            index.signal_value_encoding("top.state").unwrap(),
            FsdbValueEncoding::BitVector
        );
        assert_eq!(
            index.signal_value_encoding("top.bits").unwrap(),
            FsdbValueEncoding::BitVector
        );
        assert_eq!(
            index.signal_value_encoding("top.count").unwrap(),
            FsdbValueEncoding::BitVector
        );
        assert_eq!(
            index.signal_value_encoding("top.mystery").unwrap(),
            FsdbValueEncoding::Unsupported
        );
        assert_eq!(
            index.signal_value_encoding("top.untyped").unwrap(),
            FsdbValueEncoding::Unsupported
        );
    }

    #[test]
    fn fsdb_hierarchy_datatype_non_vectors_override_value_encoding() {
        let mut builder = FsdbHierarchyBuilder::new();
        for (idcode, kind) in [
            (11, RawDatatypeKind::Real),
            (12, RawDatatypeKind::ShortReal),
            (13, RawDatatypeKind::String),
            (14, RawDatatypeKind::Event),
        ] {
            builder.datatype(raw_datatype(idcode, kind)).unwrap();
        }
        builder.scope(scope("top", RawScopeKind::Module)).unwrap();
        for (idcode, datatype_id, name) in [
            (1, 11, "realish[3:0]"),
            (2, 12, "short_realish[3:0]"),
            (3, 13, "stringish[3:0]"),
            (4, 14, "eventish[3:0]"),
        ] {
            let mut raw = signal_with_range(idcode, name, RawSignalKind::Logic, 3, 0);
            raw.datatype_id = Some(datatype_id);
            builder.signal(raw).unwrap();
        }
        let index = builder.finish();

        for (path, kind) in [
            ("top.realish", "real"),
            ("top.short_realish", "short_real"),
            ("top.stringish", "string"),
            ("top.eventish", "event"),
        ] {
            let entry = index.resolve_signal(path).unwrap();
            assert_eq!(entry.width, 4);
            assert_eq!(
                index.signal_value_encoding(path).unwrap(),
                FsdbValueEncoding::Unsupported
            );
            assert_eq!(
                index
                    .signals_in_scope("top")
                    .unwrap()
                    .into_iter()
                    .find(|signal| signal.path == path)
                    .unwrap()
                    .kind,
                kind
            );
        }
    }

    fn scope(name: &str, kind: RawScopeKind) -> RawScopeRecord {
        RawScopeRecord {
            name: name.to_string(),
            kind,
            hidden: false,
        }
    }

    fn raw_datatype(idcode: u32, kind: RawDatatypeKind) -> RawDatatypeRecord {
        RawDatatypeRecord {
            idcode,
            kind,
            type_name: None,
            bit_width: None,
            is_signed: None,
            enum_labels: None,
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
            value_encoding: FsdbValueEncoding::BitVector,
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
            value_encoding: FsdbValueEncoding::BitVector,
        }
    }

    fn paths(entries: Vec<SignalEntry>) -> Vec<String> {
        entries.into_iter().map(|entry| entry.path).collect()
    }
}
