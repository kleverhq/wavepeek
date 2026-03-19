mod common;

use common::expr_cases::{
    ManifestEntrypoint, NegativeManifest, PositiveCase, PositiveManifest,
    expression_manifest_file_names, expression_snapshot_file_names, expression_snapshot_path,
    load_negative_manifest, load_positive_manifest, parse_negative_manifest_payload,
    parse_positive_manifest_payload, snapshot_file_name,
};

#[test]
fn shared_positive_manifest_contract_accepts_tagged_cases_and_rejects_legacy_shapes() {
    let inline_manifest = r#"
    {
      "cases": [
        {
          "kind": "event_parse",
          "name": "parse_case",
          "source": "posedge clk iff ready",
          "terms": [
            {
              "event": "posedge",
              "name": "clk",
              "iff": "ready"
            }
          ]
        },
        {
          "kind": "logical_eval",
          "name": "logical_case",
          "source": "count + 1",
          "signals": [
            {
              "name": "count",
              "ty": {
                "kind": "integer_like",
                "integer_like_kind": "int",
                "storage": "scalar",
                "width": 32,
                "is_four_state": false,
                "is_signed": true,
                "enum_type_id": null,
                "enum_labels": null
              },
              "samples": [
                {
                  "timestamp": 0,
                  "bits": "00000000000000000000000000000001",
                  "label": null,
                  "real": null,
                  "string": null
                }
              ],
              "event_timestamps": []
            }
          ],
          "timestamp": 0,
          "expected_type": {
            "kind": "integer_like",
            "integer_like_kind": "int",
            "storage": "scalar",
            "width": 32,
            "is_four_state": false,
            "is_signed": true,
            "enum_type_id": null,
            "enum_labels": null
          },
          "expected_result": {
            "kind": "integral",
            "bits": "00000000000000000000000000000010",
            "label": null,
            "real": null,
            "string": null
          }
        },
        {
          "kind": "event_eval",
          "name": "event_case",
          "source": "posedge clk",
          "tracked_signals": ["clk"],
          "signals": [
            {
              "name": "clk",
              "ty": {
                "kind": "bit_vector",
                "integer_like_kind": null,
                "storage": "scalar",
                "width": 1,
                "is_four_state": true,
                "is_signed": false,
                "enum_type_id": null,
                "enum_labels": null
              },
              "samples": [
                { "timestamp": 0, "bits": "0", "label": null, "real": null, "string": null },
                { "timestamp": 5, "bits": "1", "label": null, "real": null, "string": null }
              ],
              "event_timestamps": []
            }
          ],
          "probes": [0, 5],
          "matches": [5]
        }
      ]
    }
    "#;

    let manifest = parse_positive_manifest_payload(inline_manifest)
        .expect("inline positive cases should match the shared contract");
    assert_eq!(manifest.cases.len(), 3);

    let legacy_missing_kind = r#"
    {
      "cases": [
        {
          "name": "legacy_parse_case",
          "source": "posedge clk",
          "terms": []
        }
      ]
    }
    "#;
    assert!(
        parse_positive_manifest_payload(legacy_missing_kind).is_err(),
        "legacy positive rows without the tagged kind must be rejected"
    );

    let legacy_split_roots = r#"{ "logical_cases": [], "event_cases": [] }"#;
    assert!(
        parse_positive_manifest_payload(legacy_split_roots).is_err(),
        "legacy split positive roots must be rejected"
    );
}

#[test]
fn shared_negative_manifest_contract_enforces_host_context_and_runtime_timestamp() {
    let inline_manifest = r#"
    {
      "cases": [
        {
          "name": "parse_case",
          "entrypoint": "parse",
          "source": "(",
          "layer": "parse",
          "code": "EXPR-PARSE-EVENT-UNMATCHED-OPEN",
          "span": { "start": 0, "end": 1 },
          "snapshot": null
        },
        {
          "name": "semantic_case",
          "entrypoint": "logical",
          "source": "missing + 1",
          "layer": "semantic",
          "code": "EXPR-SEMANTIC-UNKNOWN-SIGNAL",
          "span": { "start": 0, "end": 7 },
          "snapshot": null,
          "host_profile": "integral_boolean_baseline",
          "signals": [],
          "timestamp": null
        },
        {
          "name": "runtime_case",
          "entrypoint": "logical",
          "source": "real'(xbus)",
          "layer": "runtime",
          "code": "EXPR-RUNTIME-REAL-CAST",
          "span": { "start": 0, "end": 0 },
          "snapshot": "runtime_real_cast_unknown",
          "host_profile": "custom",
          "signals": [
            {
              "name": "xbus",
              "ty": {
                "kind": "bit_vector",
                "integer_like_kind": null,
                "storage": "packed_vector",
                "width": 4,
                "is_four_state": true,
                "is_signed": false,
                "enum_type_id": null,
                "enum_labels": null
              },
              "samples": [
                { "timestamp": 0, "bits": "x101", "label": null, "real": null, "string": null }
              ],
              "event_timestamps": []
            }
          ],
          "timestamp": 0
        }
      ]
    }
    "#;

    let manifest = parse_negative_manifest_payload(inline_manifest)
        .expect("inline negative cases should match the shared contract");
    assert_eq!(manifest.cases.len(), 3);

    let legacy_missing_entrypoint = r#"
    {
      "cases": [
        {
          "name": "legacy_negative_case",
          "source": "posedge clk iff missing",
          "layer": "semantic",
          "code": "EXPR-SEMANTIC-UNKNOWN-SIGNAL",
          "span": { "start": 16, "end": 23 },
          "snapshot": null
        }
      ]
    }
    "#;
    assert!(
        parse_negative_manifest_payload(legacy_missing_entrypoint).is_err(),
        "legacy negative rows without entrypoint and host context must be rejected"
    );

    let runtime_missing_timestamp = r#"
    {
      "cases": [
        {
          "name": "runtime_case",
          "entrypoint": "logical",
          "source": "real'(xbus)",
          "layer": "runtime",
          "code": "EXPR-RUNTIME-REAL-CAST",
          "span": { "start": 0, "end": 0 },
          "snapshot": null,
          "host_profile": "rich_types_baseline",
          "signals": []
        }
      ]
    }
    "#;
    let error = parse_negative_manifest_payload(runtime_missing_timestamp)
        .expect_err("runtime negatives without timestamp must be rejected");
    assert!(error.contains("timestamp"));

    let unsupported_event_runtime = r#"
    {
      "cases": [
        {
          "name": "runtime_event_case",
          "entrypoint": "event",
          "source": "posedge clk iff real'(xbus)",
          "layer": "runtime",
          "code": "EXPR-RUNTIME-REAL-CAST",
          "span": { "start": 0, "end": 0 },
          "snapshot": null,
          "host_profile": "event_runtime_baseline",
          "signals": [],
          "timestamp": 10
        }
      ]
    }
    "#;
    let error = parse_negative_manifest_payload(unsupported_event_runtime)
        .expect_err("event-entrypoint runtime negatives should be rejected until supported");
    assert!(error.contains("event runtime diagnostics"));
}

#[test]
fn all_expression_manifests_deserialize_through_the_shared_contract() {
    for file_name in expression_manifest_file_names() {
        if file_name.ends_with("positive_manifest.json") {
            let manifest = load_positive_manifest(file_name.as_str());
            assert!(
                !manifest.cases.is_empty(),
                "manifest '{file_name}' should keep at least one case"
            );
            assert_positive_suite_ownership(file_name.as_str(), &manifest);
        } else {
            let manifest = load_negative_manifest(file_name.as_str());
            assert!(
                !manifest.cases.is_empty(),
                "manifest '{file_name}' should keep at least one case"
            );
            assert_negative_suite_ownership(file_name.as_str(), &manifest);
        }
    }
}

#[test]
fn negative_manifest_snapshots_exist_and_no_expression_snapshots_are_orphaned() {
    let mut referenced_snapshots = Vec::new();

    for file_name in expression_manifest_file_names() {
        if !file_name.ends_with("negative_manifest.json") {
            continue;
        }

        let manifest = load_negative_manifest(file_name.as_str());
        for case in manifest.cases {
            if let Some(snapshot_name) = case.snapshot.as_deref() {
                let referenced = snapshot_file_name(file_name.as_str(), snapshot_name);
                let path = expression_snapshot_path(referenced.as_str());
                assert!(
                    path.is_file(),
                    "snapshot '{}' referenced by '{}' should exist",
                    snapshot_name,
                    case.name
                );
                referenced_snapshots.push(referenced);
            }
        }
    }

    let mut snapshot_counts = std::collections::BTreeMap::new();
    for snapshot in &referenced_snapshots {
        *snapshot_counts.entry(snapshot.clone()).or_insert(0usize) += 1;
    }
    let duplicates = snapshot_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .collect::<Vec<_>>();
    assert!(
        duplicates.is_empty(),
        "every expression snapshot should be referenced at most once: {duplicates:?}"
    );

    referenced_snapshots.sort();
    referenced_snapshots.dedup();

    assert_eq!(
        expression_snapshot_file_names(),
        referenced_snapshots,
        "every expression snapshot should be referenced exactly once by the current manifests"
    );
}

fn assert_positive_suite_ownership(file_name: &str, manifest: &PositiveManifest) {
    match file_name {
        "parse_positive_manifest.json" => assert!(
            manifest
                .cases
                .iter()
                .all(|case| matches!(case, PositiveCase::EventParse(_))),
            "parse positives must contain only event_parse rows"
        ),
        "event_runtime_positive_manifest.json" => assert!(
            manifest
                .cases
                .iter()
                .all(|case| matches!(case, PositiveCase::EventEval(_))),
            "event-runtime positives must contain only event_eval rows"
        ),
        "integral_boolean_positive_manifest.json" | "rich_types_positive_manifest.json" => {
            assert!(
                manifest
                    .cases
                    .iter()
                    .all(|case| !matches!(case, PositiveCase::EventParse(_))),
                "logical/rich positives must not contain event_parse rows"
            );
        }
        other => panic!("unexpected positive manifest '{other}'"),
    }
}

fn assert_negative_suite_ownership(file_name: &str, manifest: &NegativeManifest) {
    match file_name {
        "parse_negative_manifest.json" => assert!(
            manifest
                .cases
                .iter()
                .all(|case| matches!(case.entrypoint, ManifestEntrypoint::Parse)),
            "parse negatives must contain only parse entrypoints"
        ),
        "event_runtime_negative_manifest.json" => assert!(
            manifest.cases.iter().all(|case| {
                matches!(
                    case.entrypoint,
                    ManifestEntrypoint::Parse | ManifestEntrypoint::Event
                )
            }),
            "event-runtime negatives must contain only parse or event entrypoints"
        ),
        "integral_boolean_negative_manifest.json" | "rich_types_negative_manifest.json" => {
            assert!(
                manifest
                    .cases
                    .iter()
                    .all(|case| !matches!(case.entrypoint, ManifestEntrypoint::Parse)),
                "logical/rich negatives must not contain parse entrypoints"
            );
        }
        other => panic!("unexpected negative manifest '{other}'"),
    }
}
