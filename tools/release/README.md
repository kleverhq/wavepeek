# Release Tools

This group owns release helpers used by `.github/workflows/release.yml`.

Normal workflow entrypoints:

    python3 -B tools/release/validate_tag_version.py
    python3 -B tools/release/extract_release_notes.py --version <X.Y.Z>

Focused tests:

    python3 -B -m unittest discover -s tools/release -p "test_*.py"
