#!/usr/bin/env python3
"""Offline positive/negative fixtures for check-skills.py (no third-party packages)."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import sys

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location("check_skills", Path(__file__).with_name("check-skills.py"))
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


class SkillChecks(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.skill = self.root / ".agents/skills/example/SKILL.md"
        self.skill.parent.mkdir(parents=True)
        self.skill.write_text("---\nname: example\ndescription: A fixture skill.\n---\n# Example\n", encoding="utf-8")
        self.manifest = {
            "version": 1,
            "local_max_lines": 300,
            "skills": {"example": {"owner": "local", "metadata": ["name", "description"], "requires": []}},
            "commands": [], "adapters": {}, "exceptions": [],
        }
        self.lock = {"version": 1, "skills": {}}
        (self.root / "package.json").write_text(json.dumps({"scripts": {"test": "true"}}))

    def check(self):
        (self.root / ".agents/skill-manifest.json").write_text(json.dumps(self.manifest))
        (self.root / "skills-lock.json").write_text(json.dumps(self.lock))
        return CHECKER.check(self.root)

    def append(self, text):
        self.skill.write_text(self.skill.read_text() + text)

    def upstream(self):
        entry = {"source": "fixture/upstream", "sourceType": "github", "computedHash": "a" * 64}
        self.lock["skills"]["example"] = entry
        self.manifest["skills"]["example"].update({
            "owner": "upstream", "lock": dict(entry),
            "sha256": hashlib.sha256(self.skill.read_bytes()).hexdigest(),
        })

    def test_valid_minimal(self):
        self.assertEqual(self.check(), [])

    def test_missing_resource(self):
        self.append("[guide](references/missing.md)\n")
        self.assertTrue(any("missing resource" in e for e in self.check()))

    def test_real_inline_resource(self):
        self.append("Read `references/missing.md` before editing.\n")
        self.assertTrue(any("missing resource" in e for e in self.check()))

    def test_relative_reference_document(self):
        ref = self.skill.parent / "references/guide.md"
        ref.parent.mkdir()
        ref.write_text("[main](../SKILL.md)\n")
        self.append("[guide](references/guide.md#heading)\n")
        self.assertEqual(self.check(), [])
        ref.write_text("[missing](../gone.md)\n")
        self.assertTrue(any("gone.md" in e for e in self.check()))

    def test_fenced_examples_and_placeholders(self):
        self.append("```md\n[example](missing.md)\n```\n~~~\n`references/missing.md`\n~~~\n"
                    "[template](references/<name>.md)\n![evidence]({SCREENSHOT_PATH})\n")
        self.assertEqual(self.check(), [])

    def test_missing_skill_directory(self):
        self.manifest["skills"]["missing"] = {"owner": "local", "metadata": ["name", "description"], "requires": []}
        self.assertTrue(any("missing skill directory" in e for e in self.check()))

    def test_unmanifested_skill(self):
        other = self.skill.parent.parent / "other/SKILL.md"
        other.parent.mkdir()
        other.write_text("---\nname: other\ndescription: Other.\n---\n")
        self.assertTrue(any("unmanifested skill" in e for e in self.check()))

    def test_skill_directory_without_descriptor(self):
        (self.skill.parent.parent / "incomplete").mkdir()
        self.assertTrue(any("skill directory has no SKILL.md" in e for e in self.check()))

    def test_locked_entry_not_installed(self):
        self.lock["skills"]["missing"] = {"source": "fixture/upstream"}
        self.assertTrue(any("locked skill not installed" in e for e in self.check()))

    def test_lock_metadata_and_upstream_content_preserved(self):
        self.upstream()
        self.assertEqual(self.check(), [])
        self.lock["skills"]["example"]["computedHash"] = "b" * 64
        self.assertTrue(any("lock metadata drift" in e for e in self.check()))
        self.append("Changed upstream text.\n")
        self.assertTrue(any("upstream content drift" in e for e in self.check()))

    def test_metadata_and_name(self):
        self.skill.write_text("---\nname: wrong\ndescription: []\nextra: yes\n---\n")
        errors = self.check()
        self.assertTrue(any("name must match" in e for e in errors))
        self.assertTrue(any("description" in e for e in errors))
        self.assertTrue(any("metadata fields" in e for e in errors))

    def test_duplicate_or_missing_frontmatter(self):
        self.skill.write_text("---\nname: example\nname: example\ndescription: fixture\n---\n")
        self.assertTrue(any("duplicate metadata" in e for e in self.check()))
        self.skill.write_text("# No frontmatter\n")
        self.assertTrue(any("frontmatter" in e for e in self.check()))

    def test_local_length_budget(self):
        self.append("text\n" * 300)
        self.assertTrue(any("length budget" in e for e in self.check()))

    def test_declared_package_command(self):
        self.append("Run `npm run test`.\n")
        self.manifest["commands"] = [{"skill": "example", "text": "npm run test", "cwd": ".", "kind": "package", "target": "test"}]
        self.assertEqual(self.check(), [])
        (self.root / "package.json").write_text('{"scripts": {}}')
        self.assertTrue(any("missing package command" in e for e in self.check()))

    def test_declared_script_command(self):
        self.append("Run `python3 scripts/check.py`.\n")
        self.manifest["commands"] = [{"skill": "example", "text": "python3 scripts/check.py", "cwd": ".", "kind": "file", "target": "scripts/check.py"}]
        self.assertTrue(any("missing command file" in e for e in self.check()))

    def test_stale_command_declaration(self):
        self.manifest["commands"] = [{"skill": "example", "text": "npm run test", "cwd": ".", "kind": "package", "target": "test"}]
        self.assertTrue(any("stale command declaration" in e for e in self.check()))

    def test_missing_declared_skill_and_adapter(self):
        self.manifest["skills"]["example"]["requires"] = ["specialist"]
        self.assertTrue(any("missing required skill" in e for e in self.check()))
        self.append("Use specialist through the local adapter.\n")
        self.manifest["adapters"]["specialist"] = {"skill": "example", "reason": "Specialist is not installed."}
        self.assertEqual(self.check(), [])
        self.manifest["adapters"]["specialist"]["skill"] = "absent"
        self.assertTrue(any("missing adapter skill" in e for e in self.check()))

    def test_upstream_exception_and_staleness(self):
        self.append("[guide](references/not-bundled.md)\n")
        self.upstream()
        self.manifest["exceptions"] = [{
            "skill": "example", "kind": "resource", "file": ".agents/skills/example/SKILL.md",
            "target": "references/not-bundled.md", "source": "fixture/upstream",
            "reason": "Pinned upstream documents an optional unbundled resource.",
        }]
        self.assertEqual(self.check(), [])
        ref = self.skill.parent / "references/not-bundled.md"
        ref.parent.mkdir()
        ref.touch()
        self.assertTrue(any("stale exception" in e for e in self.check()))

    def test_local_exception_is_rejected(self):
        self.manifest["exceptions"] = [{"skill": "example", "kind": "length", "target": "500", "source": "local", "reason": "Not allowed."}]
        self.assertTrue(any("upstream-only exception" in e for e in self.check()))

    def test_upstream_budget_ratchet_and_stale_exception(self):
        self.append("line\n" * 300)
        self.upstream()
        self.manifest["exceptions"] = [{"skill": "example", "kind": "length", "target": "305", "source": "fixture/upstream", "reason": "Pinned imported guide exceeds local budget."}]
        self.assertEqual(self.check(), [])
        self.skill.write_text("---\nname: example\ndescription: fixture\n---\n")
        self.upstream()
        self.assertTrue(any("stale exception" in e for e in self.check()))

    def test_reference_style_link_is_checked(self):
        self.append("Use [guide][detail].\n[detail]: references/missing.md\n")
        self.assertTrue(any("missing resource" in e for e in self.check()))

    def test_long_fence_keeps_nested_example_fenced(self):
        self.append("````markdown\n```bash\n[example](missing.md)\n```\n````\n")
        self.assertEqual(self.check(), [])

    def test_exception_replacement_must_exist(self):
        self.append("Read `assets/config.json`.\n")
        self.upstream()
        self.manifest["exceptions"] = [{
            "skill": "example", "kind": "resource", "file": ".agents/skills/example/SKILL.md",
            "target": "assets/config.json", "replacement": ".agents/references/config.json",
            "source": "fixture/upstream", "reason": "Exact resource relocated by installer.",
        }]
        self.assertTrue(any("missing exception replacement" in e for e in self.check()))
        replacement = self.root / ".agents/references/config.json"
        replacement.parent.mkdir()
        replacement.write_text("{}")
        self.assertEqual(self.check(), [])

    def test_exception_provenance_and_duplicates(self):
        self.append("[guide](references/not-bundled.md)\n")
        self.upstream()
        exception = {"skill": "example", "kind": "resource", "file": ".agents/skills/example/SKILL.md",
                     "target": "references/not-bundled.md", "source": "wrong/upstream", "reason": "Wrong source."}
        self.manifest["exceptions"] = [exception]
        self.assertTrue(any("matching provenance" in e for e in self.check()))
        exception["source"] = "fixture/upstream"
        self.manifest["exceptions"].append(dict(exception))
        self.assertTrue(any("duplicate exception" in e for e in self.check()))

    def test_stale_adapter_fails(self):
        self.append("Optional specialist advice.\n")
        self.manifest["adapters"]["specialist"] = {"skill": "example", "reason": "No consumer."}
        self.assertTrue(any("stale skill adapter" in e for e in self.check()))

    def test_bad_manifest_schema_fails_closed(self):
        self.manifest["version"] = 999
        self.assertTrue(any("unsupported schema" in e for e in self.check()))

    def test_declared_command_target_must_match_text(self):
        self.append("Run `npm run absent`.\n")
        self.manifest["commands"] = [{"skill": "example", "text": "npm run absent", "cwd": ".", "kind": "package", "target": "test"}]
        self.assertTrue(any("command target does not match" in e for e in self.check()))

    def test_path_escape_is_rejected(self):
        self.append("[outside](../../../../../../etc/passwd)\n")
        self.assertTrue(any("escapes repository" in e for e in self.check()))


if __name__ == "__main__":
    unittest.main(verbosity=2)
