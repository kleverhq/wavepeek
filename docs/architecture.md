# Architecture

## Compatibility

Identical inputs must produce identical output. Preserve ordering and repeated requests through queries and rendering. Keep outputs bounded by default. The packaged references and CLI help define the public contracts. For new waveform commands, use `--waves` for the input and require units on time arguments.

The native CLI and browser share command execution. Each invocation has its own waveform source. The browser runs it in a worker, with VCD/FST bytes from the selected browser file, to keep waveform work cancellable and local. Browser commands can use only that source.

## Waveform ownership

Ondas owns hierarchy identity and selector resolution. Preserve the selected declaration, logical signal, and explicit projection, including full-width slices. A formatted path is for output and can lose declaration or projection information if used as a query key. Keep escaped identifier terminators during CLI normalization.

WavePeek owns expression types and four-state interpretation. Keep raw expression samples and enumeration keys intact until the operation applies its own type policy.

## Range queries

Bind drivers, guards, and payload operands before opening a range query. Only drivers generate candidates. Evaluate the event against completed states at tick `t` and absolute tick `t - 1`, which can fall before the requested range. Use only callback samples within a query. A point read through the waveform facade inside the callback would reenter the reader.

Transition captures need the state at the range start even for quiet windows. AHB has a separate walker because its address and data phases can span several clock edges and wait states. It reads from dump start to preserve pending transactions and reset state at the requested lower bound.

Check for one more eligible row before reporting truncation. Stop the query on typed expression or output failures. Callback-local state limits WavePeek's own query state. The reader can load a prefix or retain format-specific state.

JSON results are all-or-error. JSONL streams rows as the query runs, so a later fatal record can follow rows already written.
