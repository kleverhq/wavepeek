# Docs helper tools

This directory contains helper scripts and tests for the GitHub Pages documentation pipeline. The stable entrypoints are the root `justfile` recipes and `.github/workflows/docs.yml`; call these scripts directly only when developing or debugging the helpers.

## Helpers

- `prepare_mkdocs.py` validates output from `wavepeek docs export`, stages Markdown for MkDocs under `tmp/docs-site/`, maps `intro.md` to `index.md`, and writes generated MkDocs navigation/config.
- `publish_docs.py` owns the publication workflow split: local `check`, no-token `stage-deploy`, and credentialed `push-staged` verification/push. It publishes exact schema artifacts from `schema/catalog.json` for every current release, leaves already deployed historical schema artifacts intact on GitHub Pages, and publishes root installer aliases only when the staged version owns `latest`. Legacy source refs without a catalog remain supported for repair flows. Packaged skills remain available through `wavepeek skill` for the installed CLI. The push path also exports the verified staged `gh-pages` tree to `tmp/docs-site/pages-artifact/` for `actions/deploy-pages`.
- `check_deploy.py` verifies deployed Pages endpoints, schema artifact availability, and optional GitHub Pages API state after publication. The API check runs only when `--repository` is provided and requires an authenticated `gh` CLI.
- `workflow_docs.py` keeps GitHub Actions glue testable: dispatch validation, release preflight, and workflow environment translation for stage/push jobs.

## Tests

Run helper tests with:

    python3 -B -m unittest discover -s tools/docs -p "test_*.py"

`just test-aux`, `just check`, and `just ci` include these tests or the docs-site check through the repository quality gates.
