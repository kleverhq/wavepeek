# Add deterministic AHB pipeline extraction

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the repository's `exec-plan` skill.

## Purpose / Big Picture

After this change, a user can run `wavepeek extract ahb` against a VCD, FST, or supported FSDB dump and receive a deterministic stream of manager-facing AHB-Lite or AHB5 pipeline events. The default stream identifies accepted address phases, real completion of previously accepted data phases, reset boundaries, and boundaries where unknown control values make prior pipeline occupancy unknowable. Optional flags expose stalled data cycles, IDLE slots, and BUSY slots without turning ordinary idle clocks into false completions.

A user can observe the result with a command such as:

    wavepeek extract ahb --waves tests/fixtures/generated/extract_ahb_lite.vcd \
      --scope top --map hclk=clk --include '^ahb_' --json

The successful JSON envelope has `command: "extract ahb"`, Issue C context, resolved canonical mappings, an `initial_data_phase` snapshot, and an ordered `events` array. Human and JSONL modes expose the same public events and preserve completion-before-address ordering when both occur on one clock edge.

## Non-Goals

This work does not support legacy full AHB split, retry, or arbitration behavior outside Arm IHI 0033. It does not provide a subordinate-facing mode using `HSELx` or `HREADYOUT`. It does not aggregate transactions, assign IDs, join address and data into one object, reconstruct bursts, count stalls, validate protocol rules, mask byte lanes, convert endianness, validate parity, or recover completions hidden before a synchronization point or by unknown control history.

## Progress

- [x] (2026-07-20 08:30Z) Read issue #67, repository guidance, current extract engines, contracts, schemas, output paths, test policy, and Arm IHI 0033C source sections.
- [x] (2026-07-20 08:30Z) Create this branch-local executable plan.
- [x] (2026-07-20 09:00Z) Add CLI and engine dispatch for `extract ahb`.
- [x] (2026-07-20 09:00Z) Implement profile parsing, source loading, signal mapping, deterministic auto-mapping, and manager-facing validation.
- [x] (2026-07-20 09:00Z) Implement the one-slot AHB clock walker, warm-up, reset/desynchronization handling, event payload validity, bounds, limits, and diagnostics.
- [x] (2026-07-20 09:00Z) Add human, JSON, and JSONL runtime output.
- [x] (2026-07-20 09:00Z) Add exact input, output, and stream contract branches and regenerate schema artifacts.
- [x] (2026-07-20 09:00Z) Add source-backed AHB-Lite/AHB5 VCD/FST fixtures and focused unit/integration/schema tests.
- [x] (2026-07-20 09:09Z) Update public docs, packaged skill guidance, architecture notes, and changelog.
- [ ] Run focused tests, `just check`, and `just ci`; resolve every failure (completed: focused engine, CLI, schema, fixture, docs, help, and skill tests; remaining: both repository gates).
- [ ] Run focused read-only reviews, apply findings, rerun affected gates, and complete an independent control review.
- [ ] Remove this WIP plan, make final conventional commits, push `feat/extract-ahb`, and open a PR that closes issue #67.

## Surprises & Discoveries

- Observation: The branch starts clean at commit `caea6c3`, which is also the current local `main`, while unreleased AXI extraction already extends schema family v2.2 even though `Cargo.toml` remains version 2.1.0.
  Evidence: `git rev-parse HEAD main` returned the same commit; current generated schema URLs are `schema-*-v2.2.json`.

- Observation: The generic extractor cannot be reused as the event engine because it counts emitted generic rows one-for-one with matching predicates and starts candidate collection at `--from`; AHB needs pre-bound warm-up and zero-to-two public rows per clock edge.
  Evidence: `src/engine/extract.rs::emit_rows` applies the limit immediately before each generic row and `run_open_plan_with_sink` binds candidate collection to the requested range.

- Observation: Source-profile aliases are a runtime convenience but current generated schemas intentionally accept canonical profile names only.
  Evidence: `ahb_lite` executes successfully through CLI/source parsing, while `schema/input.json` exposes only `ahb-lite` and `ahb5`, matching issue #67.

- Observation: JSON output carries auto-mapping warnings inside `diagnostics`; human output sends the same warnings to stderr.
  Evidence: the AHB5 fixture's `hreadyout`, `hsel`, and check decoys validate as `WPK-W0004` JSON diagnostics and as human warnings.

## Decision Log

- Decision: Use Arm IHI 0033C, Issue C, for both `ahb-lite` and `ahb5`, and report `issue: "C"`.
  Rationale: Issue C is the current specification, treats AHB as the umbrella for AHB-Lite and AHB5, and defines write strobes, signal validity, user widths, and parity/check matrices needed to define the profile boundary.
  Date/Author: 2026-07-20 / pi

- Decision: Implement a dedicated `src/engine/ahb.rs` walker and do not route unconditional clock rows through `src/engine/extract.rs`.
  Rationale: AHB completion depends on prior accepted address state, requires warm-up before `--from`, can emit two ordered rows at one edge, and must count only public events toward `--max`.
  Date/Author: 2026-07-20 / pi

- Decision: Keep the initial interface manager-facing and require global/selected `HREADY`; reject `HREADYOUT`, `HSELx`, and parity/check signals as standard mappings.
  Rationale: Arm IHI 0033C Table 2-5 and Figure 4-2 define `HREADY` as the selected progress signal presented to the manager, while `HREADYOUT` and `HSELx` are subordinate/interconnect-local signals. Mixing them makes pipeline advancement incoherent.
  Date/Author: 2026-07-20 / pi

- Decision: Reuse existing engine, contract, output, and fixture infrastructure directly, extracting only a small shared mapping helper if both AXI and AHB can use it without protocol-specific branching.
  Rationale: The repository requires the smallest correct solution. A speculative generic protocol framework would add abstractions without another concrete stateful protocol consumer.
  Date/Author: 2026-07-20 / pi

- Decision: Keep AHB mapping and schema specialization in sibling modules rather than modifying AXI internals.
  Rationale: AHB suffix decoy rules and pipeline event contracts are protocol-specific, while only three small time/limit helpers are genuinely shared with generic extraction.
  Date/Author: 2026-07-20 / pi

## Outcomes & Retrospective

Implementation has not started. The intended outcome is a complete, reviewed, pushed feature with a GitHub PR closing issue #67. This section will record the final behavior, gate evidence, review results, and any remaining gaps.

## Context and Orientation

`wavepeek` is a Rust CLI. `src/cli/extract.rs` defines the current `extract axi` and `extract generic` command-line structures. `src/cli/mod.rs` converts parsed CLI commands into `src/engine/mod.rs::Command`, which dispatches either normal collection or JSONL streaming. `src/engine/axi.rs` owns profile metadata, source JSON parsing, explicit and include-regex signal mapping, conversion into generic extraction sources, and AXI-specific output context. `src/engine/extract.rs` owns protocol-neutral edge selection and pre-edge payload sampling. AHB must be a sibling engine module because it needs persistent pipeline state.

The waveform façade is `src/waveform/mod.rs`. Expression binding and shared waveform opening live in `src/engine/expr_runtime.rs`. Time parsing and normalized rendering live in `src/engine/time.rs`, and sampled bit-vector formatting lives in `src/engine/value_format.rs`. The AHB walker should use these existing facilities rather than parse waveform files or time strings itself.

Machine output uses contract data-transfer objects rather than serializing engine structures directly. `src/contract/output.rs` defines JSON envelope data, `src/contract/stream.rs` defines JSONL context and item records, `src/contract/input.rs` defines structured input identities, and `src/contract/schema.rs` composes generated JSON Schema. AXI profile-dependent schema is isolated in `src/contract/axi_schema.rs`; AHB should use a sibling `src/contract/ahb_schema.rs`. `src/output.rs` renders human output and writes JSON/JSONL records. Canonical generated artifacts are `schema/output.json`, `schema/stream.json`, `schema/input.json`, and `schema/catalog.json`, regenerated with `just update-schema` and checked with `just check-schema`.

Source-backed waveform fixtures are Verilog under `tests/fixtures/source/`. `tools/waveform/prepare_fixtures.py`, invoked by `just prepare-waveform-fixtures`, generates ignored VCD/FST files under `tests/fixtures/generated/`. Every source and output is declared in `tests/fixtures/waveform_policy.json`. Integration tests execute the compiled CLI with helpers in `tests/common/mod.rs` and validate machine output against checked-in schemas.

The AHB pipeline contains one address acceptance slot because the current address phase overlaps the previous transfer's data phase. At each sampled rising `HCLK` edge, `HREADY=1` first completes an already pending slot and then accepts the current `HTRANS`. `NONSEQ` (`2'b10`) and `SEQ` (`2'b11`) create a pending slot. `IDLE` (`2'b00`) and `BUSY` (`2'b01`) do not create a data phase. This ordering follows Arm IHI 0033C §§3.1–3.2. A low `HREADY` extends a pending data phase. Arm IHI 0033C §5.1 defines `HRESP=1,HREADY=0` as the first ERROR-response cycle and `HRESP=1,HREADY=1` as its completing second cycle.

Every control and payload sample is taken one dump tick before the rising edge, matching existing pre-edge extraction semantics. The internal pipeline slot has five logical states: empty, pending-read, pending-write, pending-direction-unknown, and desynchronized. Pending state retains the accepted address event only so `initial_data_phase` can describe an address accepted before `--from`; normal completion rows do not repeat it.

If mapped `HRESETn` is known low, the slot clears. One `reset` row is emitted at the first sampled-low edge of each consecutive known-low episode; later low edges emit nothing. Unknown reset suppresses transfer processing, exits any reset episode, and desynchronizes. Without an earlier reset or known synchronization edge, processing starts desynchronized. A known-ready edge while desynchronized suppresses any unprovable old completion and derives the next state from current `HTRANS`. Unknown `HREADY` desynchronizes. Unknown `HTRANS` at a known-ready edge does not suppress a provable old completion, but the next state becomes desynchronized. Unknown `HWRITE` creates a pending slot with unknown direction. A transition into desynchronized emits one boundary; repeated unknown-history cycles do not repeat it.

Warm-up scans rising clock edges from dump start through event times strictly less than `--from` without emitting or counting rows. The resulting state is serialized as `initial_data_phase`: empty, desynchronized, or pending with the retained accepted address snapshot. Events at `--from` remain public because command windows are inclusive.

The default public event kinds are `address`, `data-complete`, `reset`, and `desynchronized`. `--include-stall`, `--include-idle`, and `--include-busy` independently add `data-stall`, `idle`, and `busy`. A same-edge completion is emitted before the current address/IDLE/BUSY event. Only emitted public rows count toward `--max`; a limit can split a same-edge pair and must report normal truncation. JSONL begin context and end summary count only public rows.

Address rows contain `transfer` (`nonseq` or `seq`), `direction` (`read`, `write`, or `unknown`), and mapped address/control fields. BUSY has `transfer: "busy"`, direction, and mapped address/control. IDLE has `transfer: "idle"` and only fields valid for IDLE: mapped `htrans`, `haddr`, and `hmastlock`. Reset and desynchronization rows have no transfer, direction, or payload.

Data-stall and data-complete direction comes from the pending slot. A write stall or completion can carry mapped `hwdata`, `hwstrb`, and `hwuser`; a stall always preserves mapped `hresp`. A read stall must not claim `hrdata` or successful-response fields are valid. Completion always preserves mapped `hresp`. A read completion can preserve mapped `hrdata`, including as an observation on ERROR. `hruser`, `hbuser`, and AHB5 `hexokay` are success-only and appear only when mapped `hresp` is known low. A direction-unknown data row preserves mapped read and write data observations without claiming protocol validity. Unmapped fields are omitted and X/Z values remain sized Verilog literals.

The AHB-Lite standard mapping inventory is `hclk`, optional `hresetn`, `htrans`, `hready`, `hwrite`, `haddr`, `hburst`, `hmastlock`, `hprot`, `hsize`, `hauser`, `hwdata`, `hwstrb`, `hwuser`, `hrdata`, `hruser`, `hbuser`, and `hresp`. AHB5 adds `hnonsec`, `hexcl`, `hmaster`, and `hexokay`. `hclk`, `htrans`, `hready`, and `hwrite` are required. Lowercase standard keys are used everywhere. The default profile is `ahb-lite`, `ahb_lite` is an accepted CLI/source alias, schemas accept canonical names only, and the default output name is `ahb`.

Auto-mapping follows AXI behavior: explicit mappings override include-selected candidates, includes are additive, unmatched candidates produce deterministic warnings, and ambiguous candidates fail. Normalized full-suffix matching accepts names such as `haddr`, `h_addr`, `ahb_haddr_i`, and `ahb_h_addr_i`, but must not map `hreadyout` to `hready` or check-signal suffixes to their protected signal. Scope-relative and canonical path rules remain those in `docs/public/reference/command-model.md`.

Structured source input uses exact current `$schema`, `kind: "extract.ahb.source"`, optional canonical/aliased profile, optional inclusion booleans, optional name, includes, and maps. Explicit null is rejected for typed optional fields. The source object remains extension-friendly. Source mode conflicts with CLI profile, name, map, include, and all three inclusion flags, while bounds and scope remain CLI options.

## Open Questions

There are no product questions blocking implementation. Small internal API choices will be resolved in favor of existing repository patterns and the smallest testable diff.

## Plan of Work

The first milestone adds the command shape and a compilable dedicated engine skeleton. Extend `src/cli/extract.rs` with `AhbArgs` and an `AhbProfileArg`, add `ExtractCommand::Ahb`, and route it through `src/cli/mod.rs`, `src/engine/mod.rs`, and a new `src/engine/ahb.rs`. Extend command-name and output-mode matches at the same time so all compiler exhaustiveness errors identify remaining integration points. Commit this slice only after unit-level CLI and compile checks pass.

The second milestone implements configuration and mapping. In `src/engine/ahb.rs`, define profile metadata, source-file DTOs, mappings, output context types, standard-signal inventories, parsing, required-signal validation, and deterministic include-based mapping. Reuse existing waveform resolution and diagnostic codes. Add focused unit tests for aliases, suffix tokenization, decoy rejection, duplicate/forbidden mappings, and deterministic ambiguity. Keep protocol-specific error wording in AHB code; do not turn AXI into a generic framework unless a tiny protocol-neutral helper removes exact duplication without changing behavior.

The third milestone implements the stateful event walker. Bind and resolve the required clock/control signals plus optional payload signals once. Collect rising-edge candidate times over the necessary range, sample all mapped signals at the pre-edge timestamp, and run warm-up plus public emission through one state transition function. Define explicit engine types for event kind, transfer kind, direction, pipeline state, retained accepted address, event payload values, context, and run statistics. Ensure the sink receives zero, one, or two rows per edge, starts only after warm-up context is known, and applies bounds and limit accounting to public rows only. Add direct state-transition tests and CLI tests for reset, unknowns, stalls, ERROR timing, same-edge order, final completion, warm-up, and limit splitting.

The fourth milestone integrates output and contracts. Add `CommandData::ExtractAhb`, normal and streaming context conversion, JSONL item conversion, and human rendering in `src/output.rs`. Add engine-to-contract DTOs in `src/contract/output.rs` and `src/contract/stream.rs`, source identity in `src/contract/input.rs`, and profile/event-specific schema generation in new `src/contract/ahb_schema.rs`. Schema branches must be closed for profile/event payload shapes, keep outer extension behavior, gate output-envelope optional kinds against recorded inclusion flags, and allow all AHB item event kinds in the per-record stream schema. Regenerate all four schema artifacts and add runtime/schema validation tests.

The fifth milestone adds representative end-to-end fixtures and user documentation. Create concise Verilog fixtures for AHB-Lite and AHB5 that cover zero-wait, stalls, back-to-back transfers, IDLE final completion, BUSY, ERROR, reset episodes, unknown control, sparse writes, and AHB5 fields. Declare both VCD and FST outputs in `tests/fixtures/waveform_policy.json`. Extend `tests/extract_ahb_cli.rs` and contract tests for VCD/FST parity, mapping/source validation, output modes, bounds, and schemas. Update `docs/public/commands/extract.md` with the observable pipeline FSM, `docs/public/commands/overview.md`, `docs/public/reference/machine-output.md`, workflow guidance where relevant, `docs/skills/wavepeek.md`, and `CHANGELOG.md`. Update `docs/dev/architecture.md` only if module ownership needs a new durable statement.

The final milestone validates and reviews the complete branch. Run focused tests while iterating, then `just check` and `just ci` inside the existing devcontainer. Use read-only review lanes for code correctness, architecture/performance, and docs/contracts. Apply every substantive finding, rerun impacted tests and both gates, then request a fresh control review. Remove this WIP plan before the final feature commit, ensure the branch is clean, push it to `origin`, and create a PR against `main` whose body contains `Closes #67` and summarizes behavior, tests, and protocol source.

### Concrete Steps

All commands run from `/workspaces/wavepeek/.worktrees/feat-extract-ahb`.

Create and maintain the feature in small commits:

    git status --short --branch
    just format
    just test
    git add <logical slice paths>
    git commit -m '<conventional message>'

Generate fixtures and schemas after their source contracts change:

    just prepare-waveform-fixtures
    just update-schema
    just check-schema

Run focused Rust tests during implementation, for example:

    cargo test --test extract_ahb_cli
    cargo test --test cli_contract
    cargo test --test schema_cli
    cargo test --test waveform_fixture_policy

Run the repository handoff gates:

    just check
    just ci

Expected final gate summaries are successful exits with no formatting, clippy, schema freshness, docs-site, test, or coverage failures. Optional FSDB checks may print a documented skip when the Verdi SDK is unavailable.

Push and open the linked PR only after review and a clean worktree:

    git push -u origin feat/extract-ahb
    gh pr create -R kleverhq/wavepeek --base main --head feat/extract-ahb \
      --title 'feat(extract): add AHB pipeline events' --body-file tmp/pr-67.md

The PR body must include `Closes #67`, a concise summary, and exact test commands/results.

### Validation and Acceptance

The CLI must show `ahb` under `wavepeek extract --help` and exact flags under `wavepeek extract ahb --help`. Running without `--profile` must report `profile: ahb-lite` and `issue: C`; `--profile ahb5` must expose only AHB5-valid extra mappings.

On a fixture with one accepted transfer followed by IDLE, default output must contain one `address` and one later `data-complete`, with no repeated completion on later idle clocks. On back-to-back transfers, the shared edge must contain `data-complete` before `address`. A low-ready pending phase must produce no default row and one row per stalled cycle only with `--include-stall`. IDLE and BUSY rows must likewise appear only with their independent flags.

A mapped reset held low for multiple sampled edges must produce one reset boundary and clear pending state. Unknown reset, ready, or accepted transfer history must produce one desynchronization boundary per episode. A later known-ready edge must resynchronize without inventing an old completion. Unknown payload must remain in the emitted literal and must not suppress a known event.

With `--from` inside a pending waited transfer, `initial_data_phase` must retain the pre-window accepted address and the eventual completion must appear in-range. Warm-up rows must not consume `--max`. A limit that stops between same-edge completion and address rows must emit the completion, report truncation, and produce a JSONL end count equal to emitted public items.

JSON, JSONL, and source input must validate against current generated schemas. Source files with the wrong `$schema`, wrong kind, null typed fields, forbidden profile mappings, or source/CLI conflicts must fail with empty stdout and a deterministic fatal argument error. Extension fields on the outer source object must remain accepted.

VCD and FST runs over the two source-backed fixtures must produce semantically equal JSON data. `just check` and `just ci` must both exit zero. Review must return no unresolved substantive findings. The remote PR must target `main`, use the pushed branch, and link issue #67 through `Closes #67`.

### Idempotence and Recovery

Fixture and schema generation recipes are idempotent and may be rerun. They write known generated outputs and scratch data under ignored repository locations. Do not delete unrelated files under `tmp/`. If a logical commit fails hooks, fix the reported problem and retry without bypassing hooks. If schema generation exposes an unintended public shape, revert only the current logical slice or edit contract code and regenerate; do not hand-edit generated schema snapshots.

The WIP plan is intentionally tracked during implementation and must be removed only after its final outcomes and review evidence have been captured in commit history. A failed push can be retried after checking `gh auth status` and `git remote -v`. Do not force-push or rewrite published commits unless explicitly requested.

### Artifacts and Notes

Protocol source: Arm IHI 0033C (Issue C). Relevant normative locations are §§1.3 and 3.1 for address/data overlap and `HREADY`; §3.2 and Table 3-1 for transfer encodings; §3.7 for wait behavior; §5.1 and Tables 5-1/5-2 for OKAY/ERROR response timing; §6.1 for read/write data validity; §7.1 for rising-edge sampling and reset; §§8.1–8.2 for signal validity; §10.3 for exclusive response validity; §§11.1–11.2 for user signals; and Tables A-2/A-3 for manager signal presence and check-signal exclusion.

A representative same-edge event sequence is:

    @30ns sample@29ns [data-complete read] hresp=1'h0 hrdata=32'h12345678
    @30ns sample@29ns [address nonseq write] htrans=2'h2 hwrite=1'h1 haddr=32'h00002000

A representative pending lower-bound context is:

    {
      "state": "pending",
      "address": {
        "time": "80ns",
        "sample_time": "79ns",
        "transfer": "nonseq",
        "direction": "read",
        "payload": {"htrans": "2'h2", "hwrite": "1'h0", "haddr": "32'h00001000"}
      }
    }

### Interfaces and Dependencies

Do not add dependencies. Use existing `clap`, `serde`, `serde_json`, `regex`, `schemars`, waveform backends, diagnostics, and output writers.

In `src/cli/extract.rs`, add an `AhbProfileArg` with canonical values `ahb-lite` and `ahb5`, and add `AhbArgs` containing waveform path, profile, source, name, bounds, scope, repeated maps/includes, three inclusion flags, max, absolute-path rendering, JSON, and JSONL flags.

In `src/engine/ahb.rs`, expose:

    pub fn run(args: AhbArgs) -> Result<CommandResult, WavepeekError>;
    pub fn run_jsonl<W: std::io::Write>(
        args: AhbArgs,
        writer: &mut crate::output::JsonlWriter<W>,
    ) -> Result<(), WavepeekError>;

The module must also expose crate-visible profile metadata iterators required by schema generation, analogous to `src/engine/axi.rs::profile_specs`, without exposing an unnecessary public API.

Engine output must include an AHB context type, an AHB data type with `events`, an event row type, a mapping type, a payload value type, and an initial-data-phase type. Contract DTOs must convert those types into canonical object maps so machine payload keys are lowercase standard names and human rendering can still use display paths.

The JSON command name is exactly `extract ahb`; source kind is exactly `extract.ahb.source`; default name is exactly `ahb`; default profile is exactly `ahb-lite`; both profiles report exact issue `C`.

Revision note: 2026-07-20 initial plan created after repository and protocol-source investigation. It records the complete issue #67 behavior and the implementation, validation, review, push, and PR workflow requested by the user.
