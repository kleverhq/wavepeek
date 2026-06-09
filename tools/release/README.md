# Release Tools

This group owns release helpers used by `.github/workflows/release.yml`.

Normal workflow entrypoints:

    python3 -B tools/release/validate_tag_version.py
    python3 -B tools/release/extract_release_notes.py --version <X.Y.Z>
    python3 -B tools/release/render_release_body.py --version <X.Y.Z> --repository <owner/name>
    python3 -B tools/release/publish_crate.py validate-dispatch
    python3 -B tools/release/publish_crate.py publish --source-root <release-source>

Focused tests:

    python3 -B -m unittest discover -s tools/release -p "test_*.py"
