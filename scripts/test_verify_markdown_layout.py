#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("verify_markdown_layout.py")
spec = importlib.util.spec_from_file_location("verify_markdown_layout", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class VerifyMarkdownLayoutTests(unittest.TestCase):
    def test_accepts_allowed_root_and_nested_link(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "README.md").write_text("[docs](docs/guide.md)\n", encoding="utf-8")
            (root / "docs").mkdir()
            (root / "docs/guide.md").write_text("# Guide\n", encoding="utf-8")
            self.assertEqual(module.root_layout_violations(root), [])
            self.assertEqual(module.broken_links(root), [])

    def test_reports_broken_local_link(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "README.md").write_text("[missing](docs/missing.md)\n", encoding="utf-8")
            errors = module.broken_links(root)
            self.assertEqual(len(errors), 1)
            self.assertIn("docs/missing.md", errors[0])

    def test_reports_unexpected_root_markdown(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "README.md").write_text("# ok\n", encoding="utf-8")
            (root / "audit.md").write_text("# misplaced\n", encoding="utf-8")
            self.assertEqual(module.root_layout_violations(root), ["audit.md"])

    def test_ignores_external_and_mailto_links(self) -> None:
        text = "[web](https://example.com/a.md) [mail](mailto:test@example.com)\n"
        self.assertEqual(module.local_targets(text), [])


if __name__ == "__main__":
    unittest.main()
