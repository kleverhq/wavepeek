from __future__ import annotations

import importlib.util
import pathlib
import sys
import unittest

TOOLS_DIR = pathlib.Path(__file__).parent
MODULE_PATH = TOOLS_DIR / "check_schema_contract.py"
SPEC = importlib.util.spec_from_file_location("check_schema_contract", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
check_schema_contract = importlib.util.module_from_spec(SPEC)
sys.modules["check_schema_contract"] = check_schema_contract
SPEC.loader.exec_module(check_schema_contract)


def current_catalog(path: str = "schema/output.json") -> dict[str, object]:
    return {
        "families": [
            {
                "id": "wavepeek.output",
                "version": "2.2",
                "path": path,
                "url": "https://kleverhq.github.io/wavepeek/schema-output-v2.2.json",
            },
            {
                "id": "wavepeek.stream-record",
                "version": "2.2",
                "path": "schema/stream.json",
                "url": "https://kleverhq.github.io/wavepeek/schema-stream-v2.2.json",
            },
            {
                "id": "wavepeek.input",
                "version": "2.2",
                "path": "schema/input.json",
                "url": "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json",
            },
        ],
    }


class CheckSchemaContractTests(unittest.TestCase):
    def test_validate_catalog_requires_exact_current_families(self) -> None:
        by_family = check_schema_contract.validate_catalog(current_catalog())

        self.assertEqual(
            set(by_family),
            {"wavepeek.output", "wavepeek.stream-record", "wavepeek.input"},
        )

    def test_validate_catalog_rejects_versioned_paths(self) -> None:
        with self.assertRaisesRegex(check_schema_contract.ContractError, "path"):
            check_schema_contract.validate_catalog(current_catalog("schema/wavepeek_v2.1.json"))

    def test_legacy_positional_arg_reports_just_check_schema(self) -> None:
        with self.assertRaisesRegex(check_schema_contract.ContractError, "just check-schema"):
            check_schema_contract.parse_args(["schema/wavepeek_v2.0.json"])


if __name__ == "__main__":
    unittest.main()
