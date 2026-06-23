---
id: reference/waveform-performance
title: Waveform Performance Guide
description: Understand format-level performance expectations for VCD, FST, and FSDB waveform inspection.
section: reference
see_also:
  - reference/command-model
  - commands/overview
  - commands/info
---
# Waveform Performance Guide

`wavepeek` performance depends on the waveform format, file size, selected command, requested signals, storage speed, and machine memory. The guidance below describes common expectations, not a benchmark contract.

Each waveform command opens the requested dump, executes the query, writes output, and exits. If a script runs many independent `wavepeek` commands, format setup costs are paid by each process. When command semantics allow it, prefer one command that requests the needed scopes, signals, timestamps, or time window over many tiny commands.

## Format expectations

### VCD

VCD is a textual waveform format. It is portable and widely available, but large VCD dumps can be much larger than equivalent binary formats. Queries against large VCD files may require expensive text parsing and high memory use, even when the final result is small.

For very large dumps, a narrow-looking command such as `scope`, `signal`, or `value` can still be dominated by waveform parsing rather than by printing the selected rows. Use VCD when it is the only available dump or when portability matters more than repeated-query performance.

### FST

FST is a compact, indexed waveform format. It is usually the best default for repeated inspection with `wavepeek` because metadata and selected signal data can often be loaded without reconstructing the entire dump body.

FST tends to use much less memory than large VCD input and is well suited for scripted debug loops where the same dump is queried many times.

### FSDB

FSDB is a compact proprietary waveform format read through the Synopsys Verdi FSDB Reader SDK in FSDB-enabled `wavepeek` builds. FSDB value sampling can be fast after setup, but short one-shot CLI calls may spend most of their time in native reader setup, hierarchy loading, and hierarchy teardown.

This means an FSDB file can be far smaller than a VCD file and still take seconds per independent command. The requested value lookup itself may not be the expensive step; repeated setup can dominate.

## Example order of magnitude

The table below shows one representative design dumped to three formats and queried on one machine. Treat it as scale guidance only.

| Format | Example file size | Representative one-shot command time | Peak memory |
| --- | ---: | ---: | ---: |
| FSDB | 15 MiB | 4.7-5.3 s | about 2.0 GiB |
| VCD | 3.4 GiB | 14.8-15.0 s | about 5.3 GiB |
| FST | 35 MiB | 0.2-0.7 s | about 100 MiB |

The main lesson is the ordering, not the exact numbers: large textual VCD input can be expensive, FST is usually much cheaper for repeated inspection, and FSDB one-shot commands can be dominated by setup cost.

## Conversion to FST

Converting a dump to FST can help when the converted file will be queried repeatedly. The conversion itself costs time and disk space, so it is not automatically worthwhile for one or two queries.

Conversion can also change or drop format-specific metadata. For example, hierarchy kind labels or type details may not survive every conversion path. If a workflow depends on that metadata, compare a small representative query before relying on converted output.

## Practical selection

Use the dump format you already have when you need one quick answer and conversion would take longer than the query. Prefer FST when you control dump generation or expect repeated scripted inspection. Use FSDB directly when preserving FSDB-specific metadata matters or when conversion is not available, but expect each independent CLI invocation to pay setup cost. Avoid large VCD files for repeated automation when an FST equivalent is available.
