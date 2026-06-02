# FSDB Branch Review Issues

Review scope: current `feat/fsdb` branch diff against `main` (`main...HEAD`, merge base `0d4ff45a9f6e1b5f8894493c5afa667bd9c5674c`). Per maintainer direction, `bench/e2e/runs/**` is ignored here; generated baseline JSON churn is not listed as an issue. Findings are deduplicated from reviewer agents and lightly verified against the current tree.

## Open Findings

### Medium

1. DONE `native/fsdb/wavepeek_fsdb_shim.cpp:693` — FSDB point sampling calls `ffrGotoXTag()` and then `ffrGetVC()` once, without resolving multiple value changes/glitches at the same timestamp.
   - Impact: `value`, `change`, and `property` can sample a non-final same-time value and diverge from deterministic VCD/FST behavior.
   - Suggested fix: use glitch/sequence-aware traversal to select the final value at `query_time_raw`, then add FSDB/VCD parity tests for same-time updates.

2. DONE `native/fsdb/wavepeek_fsdb_shim.cpp:1220` — datatype traversal reads only datatype block index `0` via `ffrReadDataTypeDefByBlkIdx2`.
   - Impact: FSDBs with datatype definitions in later blocks can miss enum/SV datatype records, leading to misclassified or unsupported signals.
   - Suggested fix: iterate all datatype-definition blocks, or use a traversal API that covers every block before reading the scope/var tree.

3. DONE `src/waveform/fsdb_hierarchy.rs:597` — FSDB expression metadata is incomplete: enum signals expose `EnumCore` with `enum_labels: None`, and packed/vector/enum values are hard-coded `is_signed: false`.
   - Impact: `type(state)::LABEL` can bind as enum-typed and fail later, while signed comparisons/arithmetic can disagree with VCD/FST.
   - Suggested fix: carry signedness and enum labels from FSDB datatype records into `FsdbSignalInfo`/`ExprType`; otherwise reject/document unsupported enum-label and signed-vector semantics earlier.

4. DONE `src/waveform/fsdb_hierarchy.rs:200` and `src/waveform/fsdb_hierarchy.rs:259` — duplicate normalized scope/signal paths are handled inconsistently: duplicate scopes reuse the first `scope_by_path` entry, while duplicate signals are appended but `signal_by_path` resolves only the first.
   - Impact: hierarchy traversal can collapse distinct scopes or show repeated signal paths that resolve to a different underlying idcode.
   - Suggested fix: detect/report ambiguous canonical path collisions, or preserve unique instances with unambiguous public paths; do not append duplicate signal entries that cannot be resolved consistently.

5. DONE `src/waveform/fsdb_hierarchy.rs:239`, `src/waveform/fsdb_hierarchy.rs:636`, and `src/waveform/fsdb_hierarchy.rs:671` — signal-name normalization rewrites legal local names too aggressively.
   - Impact: scalar array elements such as `mem[3]` can collapse to `mem`; escaped identifiers containing `.` or `/` can be split into synthetic scopes after `/` is rewritten to `.`.
   - Suggested fix: preserve an escaped/local-name flag and only strip bit-select suffixes or synthesize scopes when FSDB metadata proves the name is a packed bit select or flattened hierarchy artifact.

6. DONE `src/waveform/fsdb_backend.rs:206` — FSDB `expr_event_occurred` returns `Ok(false)` for non-event signals, while the Wellen backend reports an error.
   - Impact: user mistakes such as using a non-event signal as a raw event can silently produce empty `change`/`property` results instead of a diagnostic.
   - Suggested fix: return a `WavepeekError` matching the Wellen “is not a raw event” behavior before caching/querying.

7. DONE `build.rs:114` — FSDB feature builds embed the absolute local Verdi library directory as an ELF rpath.
   - Impact: feature-enabled binaries are non-reproducible/non-relocatable and leak machine-specific SDK paths.
   - Suggested fix: avoid absolute rpath by default; use documented `LD_LIBRARY_PATH`, an explicit opt-in, or a relocatable `$ORIGIN` strategy.

8. DONE `.devcontainer/initialize.sh:6` — devcontainer initialization executes the host user’s login shell to discover `VERDI_HOME`.
   - Impact: shell startup files can prompt, mutate host state, or fail inconsistently before the container starts. Lovely place for a haunted `.bashrc` to join the build.
   - Suggested fix: pass an explicit host environment value such as `${localEnv:VERDI_HOME}`/`HOST_VERDI_HOME` into mount setup instead of probing via `$SHELL -lc`.

9. DONE `tools/codex/prepare_fsdb_fixtures.sh:23` — fixture preparation unconditionally removes repository-root `vcd2fsdbLog`.
   - Impact: pre-existing user/agent data outside `tmp/` can be deleted just because it matches Verdi’s log directory name.
   - Suggested fix: run converters from a dedicated temp working directory, or only remove a log path created by this helper.

10. DECLINED `tests/fsdb_cli.rs:1361`, `tests/fsdb_cli.rs:314`, and `tools/codex/check_fsdb_env.py:11` — FSDB tests depend on `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb`, but the environment checker validates only SDK headers/libs; the smoke test also picks the first syntactically simple signal and samples at `time_start` without proving it is value-compatible.
    - Impact: otherwise-valid Verdi installs or CI images can fail for missing/changing demo fixtures or arbitrary signal ordering/value availability.
    - Suggested fix: use generated/checked fixtures for the smoke path, or make the env checker explicitly require `cpu.fsdb`; choose a known-compatible signal or probe/filter by kind, width, and value availability.

11. DECLINED `tests/fsdb_cli.rs:835` — the raw-event property parity test assumes `vcd2fsdb` preserves VCD `event` variables.
    - Impact: Verdi/version-dependent conversion can make FSDB tests fail for environment reasons rather than wavepeek behavior.
    - Suggested fix: add a converter capability check/skip for this case, or use a fixture construct whose conversion is guaranteed.

12. APPROVED `tests/fsdb_disabled_cli.rs:24` — disabled-FSDB coverage only exercises `info`.
    - Impact: regressions in `scope`, `signal`, `value`, `change`, or `property` `.fsdb` suffix handling can slip through.
    - Suggested fix: add a table-driven disabled-feature test for each command with minimal required args against a temp `.fsdb` path.

13. APPROVED `justfile:211` and `justfile:221` — e2e baseline update recipes delete committed baseline directories before the replacement run succeeds.
    - Impact: failed/interrupted runs can leave partial baseline trees ready to be committed.
    - Suggested fix: use the temp-dir/replace-after-success pattern already used by `bench-expr-update-baseline`.

14. APPROVED `README.md:8`, `README.md:27`, and `build.rs:18` — public FSDB install/support wording omits the Linux x86_64 restriction enforced by the build script.
    - Impact: users on unsupported targets can follow the public guidance and hit a build-time panic.
    - Suggested fix: document “FSDB support is currently Linux x86_64 only” wherever FSDB installation/support is advertised.

15. APPROVED `docs/public/reference/expression-language.md:138` — the expression reference still says supported operand metadata comes from VCD/FST dumps.
    - Impact: this contradicts the new FSDB-enabled `change`/`property` surface and makes FSDB expression support look unsupported.
    - Suggested fix: include FSDB-enabled builds in the wording while preserving the digital bit-vector/integral limitation and real/string caveats.

16. DECLINED `docs/tracker/wip/fsdb/cmd_property.md:186` and nearby WIP command notes — committed WIP docs include concrete Verdi Reader callback/API constants and struct member fields.
    - Impact: this conflicts with the local rule against committing Verdi headers/documentation excerpts/generated bindings and creates avoidable proprietary-content risk.
    - Suggested fix: reduce committed WIP notes to high-level implementation guidance, or keep detailed SDK notes outside the repo.

17. DECLINED `docs/tracker/wip/fsdb/arch.md:553` — FSDB architecture notes still recommend using bundled `$VERDI_HOME` example `.fsdb` files as the current fixture policy.
    - Impact: future agents can follow obsolete/partial guidance instead of the current generated fixture and RTL-artifact workflow.
    - Suggested fix: mark the section superseded by `docs/dev/testing.md`/current `just` recipes, or update it to the actual fixture contract.

### Low

18. DECLINED `native/fsdb/wavepeek_fsdb_shim.cpp:40` — `scoped_output_suppressor` redirects process-wide stdout/stderr with `dup2` around FSDB calls.
    - Impact: unrelated threads/tests writing during that window can silently lose output; failures in fd handling are also hard to reason about.
    - Suggested fix: rely on Reader suppression APIs where possible, or confine fd redirection to a checked process-wide guard with clearly bounded critical sections.

19. APPROVED `src/waveform/mod.rs:119` — `unsupported_fsdb_command_error` is an FSDB-specific hook exposed through the backend-neutral facade but currently always returns `None`.
    - Impact: stale FSDB branching remains in format-agnostic engine paths and invites future capability-gating sprawl.
    - Suggested fix: remove the hook/call sites, or replace it with a generic backend capability API only when commands actually need one.

Notes:

- The `bench/e2e/runs/**` slowdown and baseline JSON churn findings were dropped after maintainer instruction to ignore that directory.
- Rejected false positive: `xtag_type` validation was reported once, but native metadata/sample/event paths already call `require_integer_time_tags`; not listed below.
