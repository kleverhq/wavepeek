# FSDB Support RFC

## Status

Draft. Unapproved.

This work should remain in `docs/ROADMAP.md` under `Unmapped` until it is approved.
Approval should create a separate execution plan; this document is the architecture
and product-contract RFC, not the delivery plan.

## Summary

`wavepeek` should add optional FSDB support through one binary, one CLI surface,
and one optional Cargo feature: `fsdb`.

The selected direction is:

1. `cargo install wavepeek` remains the portable VCD/FST install path.
2. `cargo install wavepeek --features fsdb` is the only wavepeek-specific install
   step on supported licensed hosts.
3. `build.rs` builds an in-process native bridge from public repository sources
   against a local Verdi/FsdbReader installation referenced by `VERDI_HOME`.
4. Rust code talks only to an internal FSDB facade, not to scattered raw FFI.
5. Verdi runtime loading must be lazy and must not break VCD/FST execution when
   the runtime is unavailable.
6. Licensed development and validation require a private wrapper repo that pins
   the public `wavepeek` repo and carries private infrastructure.

Partial FSDB support is acceptable. The feature may ship command-by-command as
long as supported and unsupported cases are explicit and deterministic.

## Context

`wavepeek` is currently designed around VCD/FST only:

- `docs/DESIGN.md` lists VCD/FST as in-scope and FSDB as a future consideration.
- `docs/ROADMAP.md` keeps proprietary formats under `Unmapped`.
- CLI help text currently says `Path to VCD/FST waveform file`.
- `src/waveform/mod.rs` wraps a concrete `wellen::simple::Waveform`.
- VCD/FST parity is locked by dedicated tests such as
  `tests/change_vcd_fst_parity.rs` and `tests/property_vcd_fst_parity.rs`.
- Public quality gates currently use blanket `--all-features` checks, which would
  become incompatible with a licensed optional feature.

FSDB is not just another parser backend:

- access typically depends on Synopsys/Verdi-controlled headers, libraries, and
  environment setup
- the reader API is C/C++ oriented
- build and runtime behavior depend on host ABI and toolchain compatibility
- redistribution of vendor assets should be assumed off-limits in the public repo

## Decision Drivers

The design must satisfy these constraints:

1. Keep the default install portable and useful without Verdi.
2. Keep one user-facing binary and one CLI surface.
3. Avoid a helper daemon or per-command subprocess architecture.
4. Contain vendor-specific code behind one internal Rust API.
5. Fail clearly when FSDB support is absent, misconfigured, or only partially
   implemented.
6. Keep the public repo as the product source of truth while isolating licensed
   infrastructure and fixtures.

## Non-Goals

This RFC does not propose:

- a prebuilt public FSDB-enabled release artifact
- a public Docker image with Verdi preinstalled
- a second user-facing CLI or helper binary
- a permanent subprocess architecture for FSDB access
- a user-managed sidecar plugin as the primary install model
- redistribution of proprietary Verdi/FsdbReader assets
- day-one parity for every command or performance parity with FST
- non-Linux support in the first FSDB release

## Decision

### User-Facing Build Contract

The intended install modes are:

- `cargo install wavepeek`
- `cargo install wavepeek --features fsdb`

Contract:

- default install: VCD/FST only, no Verdi requirement
- `--features fsdb`: supported only on a documented validated matrix
- `--features fsdb`: requires `VERDI_HOME` and the expected FsdbReader layout
- `--features fsdb`: fails clearly when build prerequisites are missing or
  incompatible

There should be no separate `make fsdb-bridge` or other wavepeek-specific install
step beyond the Cargo feature.

### What `validated matrix` means

In this RFC, a validated matrix is the versioned list of exact environments for
which the FSDB build and runtime path is supported.

At minimum it should record:

- Linux distro or base image
- native compiler and standard library expectations
- Rust toolchain version
- Verdi/FsdbReader version
- required loader configuration
- required license environment assumptions

The matrix is needed because FSDB support depends on ABI and vendor-runtime
compatibility, not only on the presence of `VERDI_HOME`.

If a release advertises FSDB support, public docs must expose a public
compatibility summary for that release, including at least a matrix identifier
and the supported host/toolchain/Verdi tuple. The private wrapper repo may keep
full validation records and sign-off details private.

### Runtime Contract

Runtime behavior should be:

1. Detect waveform format.
2. Route VCD/FST to the existing `wellen` backend.
3. Route FSDB to an internal FSDB backend.
4. Initialize the FSDB bridge and resolve Verdi runtime pieces lazily.
5. If runtime resolution succeeds, execute the command.
6. If runtime resolution fails, return an actionable file/runtime error.

Even in an `fsdb` build, VCD/FST commands must continue to work without requiring
successful Verdi runtime resolution.

### Architecture

The selected layering is:

```text
wavepeek CLI / engine
    -> waveform backend seam
        -> wellen backend for VCD/FST
        -> internal Rust FSDB facade
            -> narrow C ABI
                -> in-process native bridge
                    -> Verdi/FsdbReader runtime
```

Requirements:

- all unsafe FFI stays behind one internal FSDB module
- the rest of the program uses Rust-native request/response types
- the bridge is built from public sources by `build.rs`
- proprietary headers and libraries are not checked into the public repo

### Lazy Loading Is a Gating Requirement

This design is viable only if an `fsdb`-enabled binary can avoid unconditional
Verdi loader dependencies at process startup.

That requirement is not proven yet.

A feasibility check must demonstrate both:

1. the bridge can isolate vendor runtime loading until the FSDB path is taken
2. a binary built with `--features fsdb` still runs VCD/FST commands on a host
   where Verdi runtime libraries are not available

Acceptable evidence includes both artifact inspection and runtime proof, for
example dependency inspection showing no unconditional vendor startup dependency
and a successful VCD/FST command on a host without Verdi runtime libraries.

If that cannot be demonstrated, this RFC must be revisited before further FSDB
implementation proceeds.

### Backend-Neutral Engine Contract

FSDB support requires a backend seam that is strong enough for `change` and
`property`, not only for metadata queries.

The engine-facing waveform contract must provide:

- metadata access (`time_unit`, start time, end time)
- deterministic scope traversal and signal listing using the existing canonical
  dot-separated path model
- backend-neutral signal handles with stable equality for deduplication within a
  command execution
- point sampling with clear `at or before timestamp` semantics
- previous/next timestamp access over an ordered timeline, or an equivalent query
  surface
- candidate timestamp collection for `change`
- typed value and event access for `property`
- deterministic ordering matching the existing command contracts

wellen-specific signal identity must be removed from the engine-facing API before
native FSDB logic is added.

### Partial Support Policy

FSDB support may ship incrementally.

Unsupported FSDB cases must fail deterministically and distinguish among:

- binary built without `fsdb`
- binary built with `fsdb`, but Verdi runtime unavailable
- FSDB path available, but the requested command is not implemented yet

These unsupported states must remain distinguishable through stable stderr error
classification and stable exit-code behavior.

### Format Detection and Remediation

FSDB-specific remediation should stay behind a high-confidence probe.

Rule:

- if FSDB can be identified with documented, tested high confidence, return
  differentiated FSDB guidance
- otherwise fall back to the generic unsupported-format or file-parse path

The RFC does not require an early FSDB-specific detector; it only forbids
overclaiming format certainty when confidence is low.

### Output Contract

FSDB support should preserve backend-neutral command outputs.

Normal implication:

- no mandatory schema change just because FSDB exists
- no backend-specific fields in normal output unless strictly necessary
- equivalent dumps should aim to produce equivalent command payloads

## Rejected Alternatives

The following directions are explicitly not selected as the primary model:

- helper subprocess per command
- second user-facing CLI for FSDB
- sidecar plugin installation as the main user workflow

They increase support surface, complicate packaging, and weaken the one-binary
product contract.

## Repository and Validation Model

### Public Repo Responsibilities

The public `wavepeek` repo remains the product source of truth for:

- Rust code
- the `fsdb` feature
- `build.rs`
- public bridge source
- public docs and release notes
- the canonical `Makefile`

### Private Wrapper Repo Requirement

A private wrapper repo is a requirement, not an implementation detail.

Its role is to carry licensed infrastructure that cannot live in the public repo,
including as needed:

- private fixtures or fixture manifests
- private CI definitions
- container or environment wiring
- validated-matrix records
- release sign-off artifacts for the licensed lane

The wrapper repo is an orchestration layer, not a second product repo.

### Quality-Gate Split

Once `fsdb` exists, public default lanes must remain portable.

Required policy:

- default developer and release workflows remain FSDB-off, including `make lint`,
  `make check`, `make ci`, and the ordinary public release runbook
- public lanes must not depend on blanket `--all-features`
- public docs and default developer guidance must not prescribe `--all-features`
  once `fsdb` exists
- FSDB validation runs only in an explicit licensed lane
- the licensed lane may use separate explicit targets or scripts; exact names are
  execution-plan detail
- advertising FSDB support in a release requires both the public lane and the
  licensed FSDB lane to pass their own acceptance criteria

## Acceptance Criteria

This RFC is satisfied only if the following can be demonstrated.

### Build and Packaging

- `cargo install wavepeek` works without Verdi and produces a usable VCD/FST-only
  build
- `cargo install wavepeek --features fsdb` fails clearly when `VERDI_HOME` or the
  validated matrix prerequisites are not satisfied
- the public repo does not redistribute proprietary Verdi/FsdbReader assets

### Linkage Isolation

- an `fsdb` build does not record unconditional vendor loader dependencies that
  prevent process startup on a host without Verdi runtime libraries
- the same `fsdb` build can execute a VCD/FST command successfully when Verdi is
  unavailable at runtime
- those claims are backed by repeatable dependency inspection and runtime checks
  on the validated matrix

### Runtime Behavior

- VCD/FST behavior remains compatible with the existing public contracts
- supported FSDB commands succeed on the validated matrix
- unsupported FSDB commands fail with deterministic, actionable errors

### Semantics

- engine-facing waveform APIs no longer expose `wellen`-specific signal identity
- backend-neutral output contracts remain intact unless a later RFC explicitly
  changes them
- per-command FSDB support is documented and repeated in each
  release that claims FSDB support
- the three unsupported FSDB states remain distinguishable via stable stderr
  classification and exit-code behavior

### Validation Governance

- the licensed validation lane runs outside the public default lane
- any release that advertises FSDB support references the validated matrix version
  used for sign-off
- the public release surface exposes a public compatibility summary even if the
  full sign-off artifact lives in the private wrapper repo

## Deferred to the Execution Plan

The following belong in the future execution plan, not in this RFC:

- detailed file-by-file repository changes
- phased rollout and task sequencing
- fixture generation or storage workflow details
- wrapper-repo automation details
- schedule and engineering estimates
- benchmark strategy and performance targets beyond the core runtime constraint

## Risks

1. Lazy runtime isolation may prove harder than expected or infeasible on the
   target Verdi/toolchain combination.
2. Backend abstraction may be incomplete if it models metadata queries well but
   does not close `change` and `property` semantics.
3. ABI drift across compiler, libstdc++, loader, and Verdi versions may narrow
   the supported matrix more than expected.
4. Partial command support may create user confusion unless release notes and
   errors are explicit about what is supported.

## References

### Internal

- `docs/DESIGN.md`
- `docs/ROADMAP.md`
- `docs/DEVELOPMENT.md`
- `docs/RELEASE.md`
- `Cargo.toml`
- `Makefile`
- `src/waveform/mod.rs`
- `src/waveform/expr_host.rs`
- `src/engine/expr_runtime.rs`
- `tests/change_vcd_fst_parity.rs`
- `tests/property_vcd_fst_parity.rs`

### Public

Primary sources are preferred below. DeepWiki pages are included only as secondary, illustrative documentation of a public wrapper approach.

- GTKWave supported formats: <https://gtkwave.github.io/gtkwave/intro/formats.html>
- GTKWave mailing-list note on FSDB build requirements: <https://sourceforge.net/p/gtkwave/mailman/message/37378016/>
- Synopsys Verdi product page: <https://www.synopsys.com/verification/debug/verdi.html>
- Synopsys WaveUtils / FSDB utilities overview: <https://www.synopsys.com/blogs/chip-design/verdi-waveform-utilities.html>
- `fsdb-parse` README: <https://github.com/nayiri-k/fsdb-parse/blob/main/README.md>
- `fsdb-parse` Makefile: <https://raw.githubusercontent.com/nayiri-k/fsdb-parse/main/fsdbparse/Makefile>
- `fsdb-parse` environment setup: <https://raw.githubusercontent.com/nayiri-k/fsdb-parse/main/sourceme.sh>
- `fsdb-parse` Verdi NPI wrapper: <https://raw.githubusercontent.com/nayiri-k/fsdb-parse/main/fsdbparse/npi_wrapper.py>
- `waveform_mcp` FSDB wrapper notes: <https://deepwiki.com/gokeshenzhen/waveform_mcp/7-native-fsdb-library>
- `waveform_mcp` build notes: <https://deepwiki.com/gokeshenzhen/waveform_mcp/7.1-building-libfsdb_wrapper.so>
- VaporView discussion on FSDB enablement/build friction: <https://github.com/Lramseyer/vaporview/discussions/67>
- `wave_rerunner` README note on commercial waveform formats requiring the matching simulator: <https://raw.githubusercontent.com/avidan-efody/wave_rerunner/main/README.md>
