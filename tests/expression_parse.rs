use std::collections::BTreeSet;

use wavepeek::expr::{BasicEventAst, EventExprAst, EventTermAst, parse_event_expr_ast};

mod common;
use common::expr_cases::{
    EventParseCase, NegativeCase, NormalizedTerm, PositiveCase, assert_negative_diagnostic,
    load_negative_manifest, load_positive_manifest,
};

fn load_positive_cases() -> Vec<EventParseCase> {
    load_positive_manifest("parse_positive_manifest.json")
        .cases
        .into_iter()
        .map(|case| match case {
            PositiveCase::EventParse(case) => case,
            other => panic!("parse suite only supports event_parse cases, got {other:?}"),
        })
        .collect()
}

fn load_negative_cases() -> Vec<NegativeCase> {
    load_negative_manifest("parse_negative_manifest.json").cases
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
    for case in load_positive_cases() {
        let ast = parse_event_expr_ast(case.source.as_str())
            .unwrap_or_else(|error| panic!("{} should parse: {error:?}", case.name));
        assert_eq!(normalize_ast(&ast), case.terms, "case '{}'", case.name);
    }
}

#[test]
fn parse_negative_manifest_matches_snapshots() {
    for case in load_negative_cases() {
        let diagnostic = parse_event_expr_ast(case.source.as_str())
            .expect_err(&format!("{} should fail", case.name));
        assert_negative_diagnostic("parse_negative_manifest.json", &case, &diagnostic);
    }
}

#[test]
fn parse_no_panic_corpus_holds() {
    let positive = load_positive_cases();
    let negative = load_negative_cases();

    let mut corpus = BTreeSet::new();
    for case in positive {
        corpus.insert(case.source);
    }
    for case in negative {
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
