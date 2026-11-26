# Skill Package

Edit the canonical package under `skills/wavepeek/`. The CLI embeds it and the site publishes its references. Keep metadata in the package so extraction and publication use the same values.

Use `skills/wavepeek/SKILL.md` to route readers to focused reference and usage articles. Keep these articles directly in `skills/wavepeek/references/`. Generate the CLI reference from CLI help. Keep internal links relative so they work after extraction. List each reference exactly once in `skills/wavepeek/references/docs.json` to set its place in site navigation.

Examples can simplify error handling and presentation. Keep their command syntax, output shapes, and protocol assumptions accurate.

When changing the package layout, check both skill extraction and the rendered site. Use `./dev just docs-site-check` for the local publication check.
