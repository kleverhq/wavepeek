use std::collections::BTreeSet;

use serde::Deserialize;
use wavepeek::expr::{BasicEventAst, EventExprAst, EventTermAst, Span, parse_event_expr_ast};

mod common;
use common::expr_cases::{SpanRecord, expected_layer, load_expr_manifest};

#[derive(Debug, Deserialize)]
struct PositiveManifest {
    cases: Vec<PositiveCase>,
}

#[derive(Debug, Deserialize)]
struct PositiveCase {
    name: String,
    source: String,
    terms: Vec<NormalizedTerm>,
}

#[derive(Debug, Deserialize)]
struct NegativeManifest {
    cases: Vec<NegativeCase>,
}

#[derive(Debug, Deserialize)]
struct NegativeCase {
    name: String,
    source: String,
    layer: String,
    code: String,
    span: SpanRecord,
    snapshot: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct NormalizedTerm {
    event: String,
    name: Option<String>,
    iff: Option<String>,
}

fn load_positive_manifest() -> PositiveManifest {
    load_expr_manifest("parse_positive_manifest.json")
}

fn load_negative_manifest() -> NegativeManifest {
    load_expr_manifest("parse_negative_manifest.json")
}

fn normalize_term(term: &EventTermAst) -> NormalizedTerm {
    match &term.event {
        BasicEventAst::AnyTracked { .. } => NormalizedTerm {
            event: "any_tracked".to_string(),
            name: None,
            iff: term.iff.as_ref().map(|expr| expr.source.clone()),
        },
        BasicEventAst::Named { name, .. } => NormalizedTerm {
            event: "named".to_string(),
            name: Some(name.clone()),
            iff: term.iff.as_ref().map(|expr| expr.source.clone()),
        },
        BasicEventAst::Posedge { name, .. } => NormalizedTerm {
            event: "posedge".to_string(),
            name: Some(name.clone()),
            iff: term.iff.as_ref().map(|expr| expr.source.clone()),
        },
        BasicEventAst::Negedge { name, .. } => NormalizedTerm {
            event: "negedge".to_string(),
            name: Some(name.clone()),
            iff: term.iff.as_ref().map(|expr| expr.source.clone()),
        },
        BasicEventAst::Edge { name, .. } => NormalizedTerm {
            event: "edge".to_string(),
            name: Some(name.clone()),
            iff: term.iff.as_ref().map(|expr| expr.source.clone()),
        },
    }
}

fn normalize_ast(ast: &EventExprAst) -> Vec<NormalizedTerm> {
    ast.terms.iter().map(normalize_term).collect()
}

#[test]
fn parse_positive_manifest_parses() {
    let manifest = load_positive_manifest();
    for case in manifest.cases {
        let ast = parse_event_expr_ast(case.source.as_str())
            .unwrap_or_else(|error| panic!("{} should parse: {error:?}", case.name));
        assert_eq!(normalize_ast(&ast), case.terms, "case '{}'", case.name);
    }
}

#[test]
fn parse_negative_manifest_matches_snapshots() {
    let manifest = load_negative_manifest();
    for case in manifest.cases {
        let diagnostic = parse_event_expr_ast(case.source.as_str())
            .expect_err(&format!("{} should fail", case.name));

        assert_eq!(diagnostic.layer, expected_layer(case.layer.as_str()));
        assert_eq!(diagnostic.code, case.code);
        assert_eq!(
            diagnostic.primary_span,
            Span {
                start: case.span.start,
                end: case.span.end,
            },
            "case '{}'",
            case.name
        );

        if let Some(snapshot_name) = case.snapshot.as_deref() {
            insta::assert_snapshot!(snapshot_name, diagnostic.render(case.source.as_str()));
        }
    }
}

#[test]
fn parse_no_panic_corpus_holds() {
    let positive = load_positive_manifest();
    let negative = load_negative_manifest();

    let mut corpus = BTreeSet::new();
    for case in positive.cases {
        corpus.insert(case.source);
    }
    for case in negative.cases {
        corpus.insert(case.source);
    }

    let seeds = corpus.iter().cloned().collect::<Vec<_>>();
    for source in seeds {
        corpus.insert(format!("({source}"));
        corpus.insert(format!("{source})"));
        corpus.insert(format!("{source} or"));
        corpus.insert(format!("or {source}"));
        corpus.insert(format!("{source},"));
        corpus.insert(format!("{source} iff"));
    }

    for input in [
        "(",
        ")",
        "iff",
        "or",
        ",",
        "posedge (",
        "posedge clk)",
        "posedge clk or , clk",
        "posedge clk iff (a or b",
    ] {
        corpus.insert(input.to_string());
    }

    for input in corpus {
        let guarded = std::panic::catch_unwind(|| parse_event_expr_ast(input.as_str()));
        assert!(guarded.is_ok(), "parser panicked for input: '{input}'");
    }
}
