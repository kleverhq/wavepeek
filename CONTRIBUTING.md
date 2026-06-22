# Contributing

Thanks for your interest in contributing.

The most useful contribution to this project is high-quality human feedback: reproducible bug reports, clear feature requests, edge cases, and examples from real waveform inspection workflows.

Implementation is usually not the main bottleneck. Deciding what should be built, fixed, or supported is. Maintainer review time is limited.

Before proposing work, check [GitHub Issues](https://github.com/kleverhq/wavepeek/issues), [GitHub Discussions](https://github.com/kleverhq/wavepeek/discussions), and [GitHub Milestones](https://github.com/kleverhq/wavepeek/milestones).

## Issues and discussions

The issue tracker is the maintainer work queue.

Use GitHub Issues only for reproducible bug reports and maintainer-created or maintainer-converted work items. Bug reports should use the bug report template and include current behavior, expected behavior, the shortest reproduction, impact, and environment.

Use GitHub Discussions for feature requests, ideas, questions, contribution proposals, behavior changes, design discussion, CLI or output changes, format-support requests, and direction checks.

Discussions are community input. Upvotes and comments help show demand, but they do not guarantee implementation, priority, maintainer attention, or PR approval. A maintainer may ignore a discussion, reject it, implement it directly, ask for more detail, or convert it into an accepted issue.

Issues that do not use the bug report template or are not reproducible bugs may be closed.

## Pull requests

Do not open pull requests for non-trivial changes without maintainer approval in an issue or discussion.

This applies especially to:

- new features;
- behavior changes;
- CLI or machine-output schema changes;
- broad documentation rewrites;
- large refactors;
- new dependencies.

Small fixes tied to an accepted issue are the best PR candidates. Pull requests opened without prior discussion or issue may be closed without review or left unanswered.

This is not hostility toward contributions. It avoids an asymmetry of effort: opening a PR, especially with an agent, can be cheap; reviewing, validating, integrating, and maintaining it is not.

## Development workflow

Development workflow, style, tests, automation, and release guidance live under `docs/dev/`. Run the relevant `just` gates before handoff; `just check` is the normal local gate and `just ci` is the test-inclusive gate.

## AI-generated contributions

Use tools if they help, but submit work you understand and can explain.
