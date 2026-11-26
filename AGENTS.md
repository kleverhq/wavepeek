# WavePeek development

WavePeek is a Rust CLI for deterministic waveform inspection.

Before any work, run `tree docs`. Use the filenames and tree structure to identify and read the documents relevant to the task. Apply `how-to-document.md` when working on `docs/` or comments.

Target Simplified Technical English in `docs/` and public Rustdoc.

Keep agents, Git, signing, credentials, and GitHub operations on the host. Run project tools through `./dev` and use `justfile` recipes. Run `./dev just check` before handoff and `./dev just ci` before merge.

Run the local native checks in `development-workflow.md` before opening a pull request.

Keep credentials, proprietary tools, and generated waveform data out of Git. Use ignored `tmp/` for generated outputs and logs. Remove only artifacts you own. Inspect binary dumps with waveform tools.

Update the relevant references in `skills/wavepeek/` and tests when public behavior changes.
