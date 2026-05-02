# Design Contracts Guide

This directory contains the normative semantics contracts for the CLI.

## Parent Maps

- Design docs map: `../AGENTS.md`
- Documentation map: `../../AGENTS.md`

## Source of Truth

- Documentation/help and embedded-doc semantics: `documentation_surface.md`
- Cross-cutting command semantics: `command_model.md`
- Machine output, warnings, and exit behavior: `machine_output.md`
- Expression language syntax and semantics: `expression_lang.md`
- Exact flag-level CLI surface: `../../../src/cli/`, `wavepeek -h`, `wavepeek --help`, `wavepeek help <command-path...>`, and `wavepeek docs --help`
- Machine-readable output contract: `../../../schema/wavepeek.json` and `wavepeek schema`
