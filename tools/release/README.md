# Release Tools

This group owns release-note extraction used by `.github/workflows/release.yml`.

Normal workflow entrypoint:

    python3 -B tools/release/extract_release_notes.py --version <X.Y.Z>

Focused tests:

    python3 -B -m unittest tools/release/test_extract_release_notes.py
