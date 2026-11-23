# Commands

This page helps you choose the right command family and highlights common conventions.

There are three main command categories:

- **Orientation** (`info`, `scope`, `signal`) - to get structural information about a dump
- **Inspection** (`value`, `change`, `property`, `extract`) - to get actual signal values from waveforms
- **Support** (`help`, `skill`) - to help with the tool use and integration

## Support commands

Use `wavepeek help` when you need exact syntax, defaults, required flags, and ad hoc guidance. This command works through the whole command tree: `wavepeek help` to get top level help, `wavepeek help value` to get help on main command, `wavepeek help extract axi` to get help on any subcommand. Basically, this command is an alias for `wavepeek <command-or-subcommand-or-empty> --help`.

Use `wavepeek skill` to extract the version-matched skill package. Usually, this is required once per version during installation or update. However, when the skill is unavailable or cannot be installed, this command can be used simply to materialize documentation on disk for further analysis.

## Orientation commands

Use `wavepeek info` first when you need dump metadata before running time-based queries. It reports the dump time unit and start/end bounds that other commands use. The command helps to answer: "How long is this dump?", "What is the timestep?".

Use `wavepeek scope` to explore hierarchy structure. It is the stable way to discover scope paths before narrowing later queries to a smaller part of the design. The command helps to answer: "Was this scope dumped?", "What is the exact path of this scope?".

Use `wavepeek signal` after `wavepeek scope` to inspect the signals available in a selected scope. Recursive mode broadens that view into child scopes. The command helps to answer: "Was this signal dumped?", "What is the exact name of this signal?", "What are the width and type of this signal?".

## Inspection commands

Use `wavepeek value` for exact point sampling. It is the most direct command when you already know the signal set and want values at one or more explicit timestamps. The command helps to answer: "What was the signal value at the specified time?".

Use `wavepeek change` for event-aligned signal tables or value transitions across a bounded time range. It comes with a trigger selection (event expressions) and several modes of table organization. The trigger can be synchronous or asynchronous (more in [Clocks and sampling](sampling.md)). The command helps to answer: "Did these signals change during this range?", "What were the values of these signals on every clock tick?".

Use `wavepeek property` when you want to evaluate a logical expression on event-selected timestamps instead of printing raw signal snapshots. It uses event expressions to describe the evaluation trigger and boolean expressions to describe predicates (more in [Boolean conditions](predicates.md)). Its capture modes control whether you keep every match or only state transitions such as asserts and deasserts. The command helps to answer: "Were there any points when this signal took the specified value?", "Were there any requests or acknowledgements?", "How many times did this condition become true?".

Use the `wavepeek extract` command family when you want to find synchronous events and sample payload values using RTL "pre-edge" semantics. The `extract generic` subcommand covers custom handshakes, FIFO pushes and pops, and other transfer-like events using request or validity signals.

Use `wavepeek extract ahb` to work with the AMBA AHB protocol family: manager-facing AHB-Lite or AHB5. It tracks address/data pipeline events, including delayed completions and synchronization boundaries.

Use `wavepeek extract apb` to work with the AMBA APB protocol family: APB3, APB4, and APB5. It tracks Setup, waited Access, and completed Access states and provides values of all sideband signals during these states.

Use `wavepeek extract atb` to work with the AMBA ATB protocol family and capture transfer, flush, and synchronization-request events.

Use `wavepeek extract axi` to work with the AMBA AXI protocol family: AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM and ACE5-LiteACP. It tracks all ready/valid channel transfers and provides rows with captured metadata.

Use `wavepeek extract axistream` to work with the AMBA AXI-Stream protocol family: AXI4-Stream or AXI5-Stream. It captures all ready/valid transfers, including an explicit mode for a physically omitted ready signal.

## Human-readable and machine-readable modes

Waveform commands default to human-readable output. Machine-readable output is enabled explicitly with `--json` for a complete JSON value or `--jsonl` for a newline-delimited stream of records.

Human-readable output is optimized for compact operator use and may vary when formatting improves. Use `--json` when a client wants one complete result. Use `--jsonl` when a client wants to consume waveform rows incrementally.

`--path-style logical|sv` selects path spelling for `scope`, `signal`, `value`, `change`, and each `extract` subcommand. It applies to all output modes. Filters and includes use logical spellings. See [Path and scope resolution](paths.md).

## Bounded Output

`wavepeek` is designed to avoid flooding terminals and LLM context windows. Commands therefore keep output bounded by default through one or more of these mechanisms:

- explicit count limits such as `--max`,
- depth limits such as `--max-depth`,
- the finite size of the requested input set.

When a command truncates output because of an active limit, it emits a warning diagnostic. `change`, `property`, and `extract` use `--max` for event-row limits.

Every successful command with numeric or unlimited `--max` reports `complete`, `returned`, `limit`, and `total` in machine output. Unlimited `--max` is represented by `limit: null`. A numeric result becomes incomplete only after the command finds another matching public item beyond the limit. Reaching the selected result-set end makes `total` exact; otherwise `total` remains unknown unless the command already collected the full selected set. `--max unlimited` completes the scan and reports an exact total. Depth options such as `--max-depth` define the selected set and do not count as incomplete results.

Pass `--summary` when only completeness metadata, optional command context, and diagnostics are needed. The command performs the same selection, filtering, limit checks, and early termination but suppresses result rows. Human mode prints the four summary fields; JSON omits `data`; JSONL emits no `data` records and places the summary in its terminal `end` record.

`change` applies its `--max` limit after row-mode filtering. Dense mode counts every selected event that can be sampled; sparse mode counts only selected samples whose requested values changed from the previous emitted sample. Sparse mode starts from the values at `--from` and does not emit that initial state. Delta rows include all values in the first row, then only values changed since the previous emitted row.

## Deterministic Ordering

Deterministic output is a repository-wide design requirement. Given identical input data and identical command arguments, `wavepeek` must emit results in a stable order.

The list-valued `value --at`, `value --signals`, `change --signals`, and `extract generic --payload` options accept comma-separated values, repeated options, or both. Mixed forms flatten in command-line order and preserve duplicates.

The main ordering rules are:

- `scope` traverses hierarchy in pre-order depth-first order with lexicographic child ordering.
- Recursive `signal` queries walk scopes in that same stable order and sort signals deterministically within each visited scope.
- `value` preserves the request order from `--at` and `--signals`, including duplicates.
- `change` preserves `--signals` request order and duplicates within each full row or changed subset, and emits rows in ascending normalized timestamp order.
- `property` emits rows in ascending normalized timestamp order.
- `extract generic` preserves payload request order and duplicates, and emits rows in ascending event timestamp order and, when multiple sources match at the same timestamp, source declaration order.
- When multiple diagnostics apply, their order is deterministic for a given command contract.
