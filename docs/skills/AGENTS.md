# Packaged Skill Guidance

## Source of Truth

- Packaged wavepeek skill: `wavepeek.md`
- Public docs topic corpus: `../public/`
- Skill runtime surface: `../../src/docs/mod.rs` and `../../src/engine/skill.rs`

## Local Guidance

Keep the skill short and routing-oriented. It may preserve critical safety rules, but it should send agents to `wavepeek help`, `wavepeek docs topics`, and public reference topics instead of duplicating detailed command contracts.
