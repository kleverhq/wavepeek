# Packaged Skills Guide

This directory contains packaged agent-facing skill Markdown emitted by `wavepeek docs skill`.

## Parent Map

- Documentation map: `../AGENTS.md`

## Source of Truth

- Packaged wavepeek skill: `wavepeek.md`
- Public docs topic corpus: `../public/`
- Skill runtime surface: `../../src/docs/mod.rs` and `../../src/engine/docs.rs`

Keep skills short and routing-oriented. They may preserve critical safety rules, but should point agents to `wavepeek help`, `wavepeek docs topics`, and public reference topics instead of duplicating detailed command contracts.
