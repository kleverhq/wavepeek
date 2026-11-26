# How to Document WavePeek

Write for an engineer who knows Rust, EDA, and waveform formats. Record decisions and their reasons, assumptions, invariants, and limits that would be hard to infer from the code.

Keep each fact in one place. Use `justfile`, workflow files, tests, and CLI help for details they already define. Name the relevant source or recipe when it helps the reader find it. Keep user-facing CLI semantics in `skills/wavepeek/references/`.

Use comments for local constraints and choices whose reasons are unclear at the point of use. Use public Rustdoc for caller obligations, guarantees, and errors.

When revising a document, check its claims against the project and remove stale or redundant text. Delete the file if its remaining content belongs elsewhere.

Choose descriptive filenames so `tree docs` helps readers find the right document. Refer to unambiguous documents by filename in backticks. Use a repository-root-relative path when the filename is ambiguous or the target is outside `docs/`.

Use short sentences and precise terms. Keep Markdown paragraphs and list items on one source line. Use tables when requested.
