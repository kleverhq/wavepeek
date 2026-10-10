# Provision Ondas fixtures in the development image

This ExecPlan follows the `exec-plan` skill and is maintained as work proceeds.

## Purpose / Big Picture


Development, CI, and benchmarks will use eight selected waveforms from
`kleverhq/ondas-fixtures`. A Docker build downloads and verifies them once;
tests and benchmarks use the installed files without network access. Running
`./dev just test` and the benchmark smoke recipes demonstrates the migration.

## Non-Goals


Keep the existing waveform selection and CLI behavior. Do not add a submodule,
install the complete upstream corpus, change Rust dependencies, or bake derived
VCD and FSDB files into the image. VCD is a text waveform format; FST and FSDB
are binary waveform formats. Existing local small-fixture generation remains.

## Progress


- [x] (2026-10-10) Inspected provisioning, fixture consumers, and upstream installer.
- [x] (2026-10-10) Created `chore/ondas-fixtures` from `d82f7ba977b7ed71d5f739ac92f85faada26c05c` and installed host hooks.
- [x] (2026-10-10) Implemented pinned Docker provisioning and native fixture paths.
- [x] (2026-10-10) Adapted VCD/FSDB helpers, smoke selection, catalogs, and existing helper tests.
- [x] (2026-10-10) Updated current documentation and packaged quickstart.
- [x] (2026-10-10) Passed formatting and all auxiliary suites, including 22 FSDB helper tests.
- [x] (2026-10-10) Passed default and FSDB clippy with warnings denied.
- [x] (2026-10-10) Built the cached validation image, refreshed the private container, and verified all eight inputs against metadata and CLI parsing; the clean build hit a PyPI timeout.
- [x] (2026-10-10) Passed 1,788 Rust tests across 40 suites, including 24 FSDB tests.
- [x] (2026-10-10) Passed FST/FSDB smoke, `just check`, and `just ci`; source coverage exceeds all 90% thresholds.
- [x] (2026-10-10) Passed `just pre-commit` on all files and `just dev-setup` with declared tool versions.
- [ ] Run OCR with its maximum supported effort, resolve material findings, and commit.

## Surprises & Discoveries


Ondas releases contain incremental parts of the corpus. All eight selected
payloads are published in `v4.1.1`; the latest release does not include them.
The upstream installer searches release assets by fixture ID and SHA-256,
then verifies the unpacked size and checksum. SHA-256 is the content digest.
Its selected-install command uses Python's standard library only.

Every selected payload has a different SHA-256 from the old `rtl-artifacts`
`v1.0.0` payload. Tests must verify expected hierarchy and signals rather than
assume byte identity. Native fixture filenames are all `waveform.fst`, so
FSDB smoke selection must match fixture directory IDs rather than basenames.

The ignored `.env` selects `.devcontainer.local/devcontainer.json`, whose
Dockerfile derives from `wavepeek-dev:bootstrap-public-20261010`. Rebuilding
only the public profile does not refresh that private base image.

The clean public build failed while pip downloaded Playwright from PyPI:
`ReadTimeoutError` canceled the other build stage after it had downloaded and
verified five Ondas fixtures. A validation image uses the existing tool
filesystem, a Git archive of the exact pinned metadata, and eight local cached
payloads. The same upstream installer verifies those payloads during the
temporary build. This validates fixture installation and consumers, but does
not count as a successful clean network build. Temporary build inputs and logs
live under `tmp/ondas-fixtures/`; the tracked Dockerfile keeps network downloads.

OCR's default selection excludes Markdown and helper tests. A run-local
`tmp/ondas-fixtures/ocr-rule.json` includes them. Three full benchmark catalogs
exceed OCR's default per-file prompt limit; compare their entire text against
the baseline after applying only the eight known path substitutions. The
smoke catalog receives both this comparison and OCR review.

## Decision Log


Decision: Pin metadata and the installer to upstream commit
`aadb6d00597265fdef1a8227825465dbffecee43` and reuse its selected installer.
Rationale: This avoids implementing release discovery, decompression, and
checksum verification twice. Date/author: 2026-10-10, Codex.

Decision: Install only the eight existing scenarios under
`/opt/ondas-fixtures/fst/<fixture-id>/waveform.fst`.
Rationale: Native paths preserve fixture identity and allow derived
`waveform.vcd` and `waveform.fsdb` neighbors. Copy selected directories and
upstream license texts into the final image with owner `ubuntu`.
Date/author: 2026-10-10, Codex.

Decision: Include tests and Markdown in OCR using a run-local rule, and audit
the three oversized catalogs through exact baseline comparison.
Rationale: Their changes are mechanical path substitutions; a complete text
comparison detects unrelated edits without sending unchanged large signal
lists to the model. Date/author: 2026-10-10, Codex.

## Outcomes & Retrospective


Provisioning, consumers, conversion helpers, catalogs, and documentation now
use native Ondas paths. Formatting, all auxiliary suites, and default/FSDB
clippy pass. Exact baseline comparison confirms that the four benchmark
catalogs contain only known path substitutions (156 scenarios in each full
catalog, 14 in the smoke catalog). The cached validation image contains exactly
eight selected FST files with verified size, SHA-256, writable directories, and
successful CLI parsing. It contains no old fixture directory or environment
variable. All 1,788 Rust tests pass, including 24 FSDB tests. FST/FSDB smoke,
`just check`, and `just ci` pass. Coverage is 91.86% regions, 91.66% functions,
and 93.01% lines. All-file pre-commit and tool setup pass. OCR remains; the clean
network build limitation is recorded above.

## Context and Orientation


`.devcontainer/env_contract.sh` declares tool versions and fixture locations.
`.devcontainer/Dockerfile` downloads pinned upstream metadata and invokes its
installer for eight selected files in a separate Docker build stage, then
copies them into the final image.
`justfile` checks installed files before tests, coverage, and benchmarks.
`tests/common/mod.rs::ondas_fixture_path` resolves external integration inputs.
`bench/e2e/tests.json` and `bench/e2e/tests_commit.json` contain absolute input
paths. `tools/fsdb/generate_bench_catalog.py` derives VCD/FSDB catalogs by
replacing waveform suffixes. `tools/fsdb/prepare_fsdb_fixtures.sh` converts
external FST files, and `tools/fsdb/check_fsdb_bench_artifacts.py` verifies
the converted paths required by a catalog. The corresponding Python tests
exercise conversion selection, persistent-VCD reuse, and missing inputs.

Use the following mapping. Each right-hand directory is relative to
`ONDAS_FIXTURES_DIR` and contains `waveform.fst`:

    picorv32_test_vcd.fst -> fst/fst0013-picorv32-test-vcd
    picorv32_test_ez_vcd.fst -> fst/fst0012-picorv32-test-ez-vcd
    scr1_max_axi_coremark.fst -> fst/fst0022-scr1-max-axi-coremark
    scr1_max_axi_isr_sample.fst -> fst/fst0025-scr1-max-axi-isr-sample
    scr1_max_axi_riscv_compliance.fst -> fst/fst0027-scr1-max-axi-riscv-compliance
    chipyard_DualRocketConfig_dhrystone.fst -> fst/fst0006-chipyard-dualrocketconfig-dhrystone
    chipyard_ClusteredRocketConfig_dhrystone.fst -> fst/fst0000-chipyard-clusteredrocketconfig-dhrystone
    chipyard_ClusteredRocketConfig_mt-memcpy.fst -> fst/fst0002-chipyard-clusteredrocketconfig-mt-memcpy

## Open Questions


Resolve any fixture incompatibility through existing integration tests and
benchmark preflight. OCR help confirms `high` is its maximum effort preset.
No user input is required for routine implementation.

## Plan of Work


### Milestone 1: Install and consume pinned fixtures


Replace the old fixture version, root, and file list in
`.devcontainer/env_contract.sh` with `WAVEPEEK_ONDAS_FIXTURES_REV`,
`ONDAS_FIXTURES_DIR`, and `WAVEPEEK_ONDAS_FIXTURES`. The list contains the
eight relative directories above. The Docker fixture stage downloads the
pinned repository archive, runs `python3 -B scripts/install.py` with that list,
and copies only selected fixture directories and `LICENSES` into its output.
The final stage exports the new root and copies that output as `ubuntu`.

Rename fixture recipes and their callers in `justfile`,
`tools/bench/capture.py`, and `tools/bench/test_gate.py`. Resolve test inputs
through `ondas_fixture_path(fixture: &str) -> PathBuf`, joining the new root,
relative fixture directory, and `waveform.fst`. Replace all eight external
paths in the FST and smoke benchmark catalogs. After rebuilding, tests must
find the new files and continue to expose their expected hierarchy.

### Milestone 2: Preserve conversion and smoke behavior


Update `tools/fsdb/prepare_fsdb_fixtures.sh` to select explicitly declared
fixture directories and match its existing RTL filter against directory IDs.
Preserve test-VCD-only mode, converter isolation, and persistent-VCD reuse.
Update `tools/fsdb/check_fsdb_bench_artifacts.py` to recognize native nested
external paths. Missing required converted files must still fail. Adapt
existing Python tests to nested directories and equal waveform basenames.
Update the existing FSDB smoke filter to select only PicoRV32 EZ and SCR1 AXI
compliance. Regenerate VCD and FSDB catalogs through root recipes; their
paths must match the converter outputs. Focused helper tests must pass.

### Milestone 3: Document, verify, and review


Update `.devcontainer/AGENTS.md`, `docs/environment.md`, `docs/architecture.md`,
`docs/benchmarking.md`, `docs/fsdb.md`, and
`skills/wavepeek/references/quickstart.md`. Keep historical changelog entries.
Rebuild the public image, refresh the ignored private profile's base tag,
and recreate only this worktree's selected private container. Run the gates
below. Run OCR over the complete branch diff with migration context and its
maximum supported effort; retain the report under `tmp/ondas-fixtures/`.
Investigate findings and fix material defects within this migration's scope.

### Concrete Steps


Run host Git, Docker lifecycle, and OCR commands from
`/home/esynr3z/projects/wavepeek`. Run development tools through `./dev` and
root recipes. After edits:

    ./dev just format
    ./dev just update-bench-e2e-fsdb-catalog
    ./dev just update-bench-e2e-vcd-catalog
    WAVEPEEK_DEV_CONFIG=.devcontainer/devcontainer.json ./dev --recreate true
    devcontainer build --workspace-folder "$PWD" --config .devcontainer/devcontainer.json --image-name wavepeek-dev:ondas-fixtures

Point the ignored private Dockerfile's `WAVEPEEK_PUBLIC_IMAGE` default at the
new tag, then run:

    ./dev --recreate true
    ./dev just test-aux
    ./dev just test
    ./dev just bench-e2e-smoke-commit
    ./dev just check
    ./dev just ci
    ./dev just pre-commit
    ocr review --help
    ocr review --audience agent --effort high --rule tmp/ondas-fixtures/ocr-rule.json --from d82f7ba977b7ed71d5f739ac92f85faada26c05c --to HEAD --background "Migrate eight waveform inputs to pinned Ondas fixtures downloaded during Docker builds; preserve nested path checks, VCD/FSDB conversion, smoke selection, and CLI behavior with KISS/YAGNI. Three oversized catalogs are verified by exact baseline comparison allowing only known fixture-path substitutions." --output tmp/ondas-fixtures/ocr-review.txt

Use the effort value confirmed by help if a higher value is supported. Commit
implementation before range review so `HEAD` includes the complete change.
Review subsequent fixes in workspace mode and rerun affected checks.

### Validation and Acceptance


All eight selected `waveform.fst` files must exist under the new root in a
fresh public image; each must match its installed `fixture.json` size and
SHA-256. No other upstream waveforms should be installed. The selected
directories must be writable by `ubuntu` for derived VCD/FSDB neighbors.
Integration tests must retain PicoRV32 parameters, SCR1 pipeline scopes, and
the packed `q_err` signal. The FST smoke and optional FSDB smoke must run
successfully. Helper tests must prove that nested missing FSDB files fail,
filtered conversion omits unselected fixtures, and persistent VCD is reused.
All standard gates must pass; OCR must cover the complete change and leave
no unresolved material findings. Active provisioning and consumers must
contain no old repository, environment variable, or `/opt/rtl-artifacts` path.

### Idempotence and Recovery


The pinned installer verifies existing matching files and fails on missing or
corrupt assets. Rebuilding retries provisioning without changing the pin.
Docker recreation affects only the selected worktree/configuration container.
The existing private container may remain running while the public image
builds. Do not prune unrelated Docker data or clear `tmp/`. Keep this plan for
branch handoff and remove it before merge unless the maintainer retains it.

### Artifacts and Notes


The baseline is `d82f7ba977b7ed71d5f739ac92f85faada26c05c`. Build and gate logs,
fixture validation evidence, and OCR output belong in `tmp/ondas-fixtures/`.
Record concise results here as milestones finish.

### Interfaces and Dependencies


Use the existing Ubuntu base, `curl`, Python 3, Icarus/GTKWave tools, and
optional Verdi tools. The pinned upstream installer uses only Python's
standard library and GitHub's public release APIs. Runtime recipes require
`ONDAS_FIXTURES_DIR`, exported by the image and root `justfile`. No Cargo
dependency changes are required.

Create the run-local OCR rule file before review with this content:

    {
      "include": ["**/*"],
      "rules": [{
        "path": "**/*",
        "rule": "Verify pinned build-only provisioning, nested fixture paths, conversion selection, persistent VCD reuse, tests, and documentation. Follow KISS/YAGNI and report concrete defects.",
        "merge_system_rule": true
      }]
    }

Revision note (2026-10-10): Created the plan from inspected local consumers and
the pinned upstream installer; recorded the private-image refresh requirement.

Revision note (2026-10-10): Completed implementation and auxiliary validation;
confirmed OCR's maximum effort and started fresh public-image provisioning.

Revision note (2026-10-10): Expanded OCR selection to tests and documentation;
recorded exact baseline audit for oversized path-only benchmark catalogs.

Revision note (2026-10-10): Recorded the clean-build PyPI timeout and selected
cached pinned inputs for independent image and consumer validation.

Revision note (2026-10-10): Verified the image contract and all Rust/FSDB tests.
Reused local Cargo registry caches; the successful test command used
`./dev env CARGO_NET_OFFLINE=true just test` with the unchanged lockfile.

Revision note (2026-10-10): Passed FST/FSDB smoke and both standard gates;
recorded source coverage and started the all-file pre-commit gate.

Revision note (2026-10-10): Completed all-file pre-commit and verified declared
tool versions in the new private container; ready for commit and range review.
