#!/usr/bin/env python3
"""Offline skill coherence checks. Contract/schema: agents-docs/SKILL_TEMPLATE.md.

Only the small frontmatter subset actually used here is parsed, not arbitrary
YAML. Commands are declarations checked against files/package scripts; they are
never executed. Upstream installer hashes are compared as opaque lock metadata.
"""
import hashlib
import json
from pathlib import Path
import re
import sys
from urllib.parse import unquote


ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ".agents/skill-manifest.json"


def unfenced(text):
    """Exclude fenced examples, respecting fence character and closing length."""
    fence = None
    lines = []
    for line in text.splitlines():
        match = re.match(r"^\s{0,3}(`{3,}|~{3,})(.*)$", line)
        if fence:
            if match and match[1][0] == fence[0] and len(match[1]) >= len(fence) and not match[2].strip():
                fence = None
        elif match:
            fence = match[1]
        else:
            lines.append(line)
    return "\n".join(lines)


def metadata(text):
    lines = text.splitlines()
    if not lines or lines[0] != "---" or "---" not in lines[1:]:
        raise ValueError("missing or unterminated frontmatter")
    fields = {}
    current = None
    for line in lines[1:lines.index("---", 1)]:
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        match = re.match(r"^([\w-]+):\s*(.*)$", line)
        if match:
            current, value = match.groups()
            if current in fields:
                raise ValueError(f"duplicate metadata: {current}")
            fields[current] = value
        elif line.startswith("  ") and current:
            fields[current] += " " + line.strip()
        else:
            raise ValueError(f"unsupported frontmatter line: {line}")
    return fields


def resources(text):
    """Find real Markdown links and explicitly named bundled resources in prose."""
    text = unfenced(text)
    links = set(re.findall(r"!?\[[^\]]*\]\(([^\s)]+)(?:\s+[^)]*)?\)", text))
    links.update(re.findall(r"^\s{0,3}\[[^\]]+\]:\s*(\S+)", text, re.MULTILINE))
    # Prose also names bundle resources without Markdown links. This deliberately
    # does not infer arbitrary code filenames or generated paths from examples.
    inline = set(re.findall(r"(?<![\w/])((?:references|templates|scripts|assets)/[\w./-]+\.[\w]+)", text))
    return [(ref, False) for ref in sorted(links)] + [(ref, True) for ref in sorted(inline - links)]


def placeholder(ref):
    return any(ch in ref for ch in "<>{}*$") or ref in {"link", "path", "url", "..."}


def check(root):
    root = Path(root).resolve()
    errors = []
    try:
        manifest = json.loads((root / MANIFEST).read_text(encoding="utf-8"))
        lock = json.loads((root / "skills-lock.json").read_text(encoding="utf-8"))
        if manifest["version"] != 1 or lock["version"] != 1 or manifest["local_max_lines"] != 300:
            raise ValueError("unsupported schema or local length budget (must remain 300)")
        skills, locked = manifest["skills"], lock["skills"]
        if not isinstance(skills, dict) or not isinstance(locked, dict) or not skills:
            raise ValueError("skills must be a nonempty ownership map")
        exceptions = manifest["exceptions"]
        adapters = manifest["adapters"]
        commands = manifest["commands"]
        if not isinstance(exceptions, list) or not isinstance(adapters, dict) or not isinstance(commands, list):
            raise ValueError("exceptions/commands must be lists; adapters must be an object")
    except (OSError, ValueError, KeyError, TypeError) as exc:
        return [f"manifest/lock unreadable: {exc}"]

    def report(message):
        errors.append(message)

    def inside(path):
        return path.resolve().is_relative_to(root)

    skill_root = root / ".agents/skills"
    installed = {p.parent.name: p for p in skill_root.glob("*/SKILL.md")}
    for directory in sorted(skill_root.iterdir()):
        if directory.is_dir() and not (directory / "SKILL.md").is_file():
            report(f"skill directory has no SKILL.md: {directory.name}")
    for name in installed.keys() - skills.keys():
        report(f"unmanifested skill: {name}")
    for name in skills.keys() - installed.keys():
        report(f"missing skill directory: {name}")
    for name in locked.keys() - installed.keys():
        report(f"locked skill not installed: {name}")

    used_exceptions = set()
    valid_exceptions = {}
    for index, exc in enumerate(exceptions):
        if not isinstance(exc, dict):
            report(f"invalid exception: {index}")
            continue
        name = exc.get("skill")
        spec = skills.get(name, {})
        if (spec.get("owner") != "upstream" or not exc.get("reason")
                or exc.get("source") != locked.get(name, {}).get("source")):
            report(f"upstream-only exception lacks matching provenance/reason: {name}")
            continue
        kind = exc.get("kind")
        if kind not in {"length", "resource"} or not isinstance(exc.get("target"), str):
            report(f"invalid exception kind/target: {name}")
            continue
        if kind == "resource" and not exc.get("file", "").startswith(f".agents/skills/{name}/"):
            report(f"resource exception must name one upstream file: {name}")
            continue
        key = (name, kind, exc.get("file", ""), exc["target"])
        if key in valid_exceptions:
            report(f"duplicate exception: {key}")
        replacement = exc.get("replacement")
        if replacement and (not inside(root / replacement) or not (root / replacement).is_file()):
            report(f"missing exception replacement: {name}: {replacement}")
            continue
        valid_exceptions[key] = index

    def exempt(name, kind, file, target):
        key = (name, kind, file, target)
        index = valid_exceptions.get(key)
        if index is None:
            return False
        used_exceptions.add(index)
        return True

    texts = {}
    required = set()
    for name, spec in skills.items():
        if not isinstance(spec, dict) or spec.get("owner") not in {"local", "upstream"}:
            report(f"invalid ownership: {name}")
            continue
        path = installed.get(name)
        if path is None:
            continue
        text = path.read_text(encoding="utf-8")
        texts[name] = text
        if spec["owner"] == "upstream":
            if name not in locked or spec.get("lock") != locked[name]:
                report(f"lock metadata drift: {name}")
            if spec.get("sha256") != hashlib.sha256(path.read_bytes()).hexdigest():
                report(f"upstream content drift: {name}; re-import deliberately, never silently patch")
        elif name in locked:
            report(f"locked skill cannot be locally owned: {name}")
        try:
            fields = metadata(text)
            if fields.get("name", "").strip("\"'") != name:
                report(f"name must match skill directory: {name}")
            description = fields.get("description", "").lstrip(">|-+ ").strip("\"'")
            if not description or description.startswith(("[", "{")):
                report(f"description must be nonempty text: {name}")
            if set(fields) != set(spec.get("metadata", [])):
                report(f"metadata fields differ from declared schema: {name}")
        except ValueError as exc:
            report(f"{name}: {exc}")
        count = len(text.splitlines())
        if count > manifest["local_max_lines"]:
            # Exact imported line count, not an expandable budget. Under-budget
            # imports no longer consume their exception and fail the stale check.
            if not exempt(name, "length", "", str(count)):
                report(f"length budget exceeded: {name} ({count} > 300)")
        deps = spec.get("requires")
        if not isinstance(deps, list) or not all(isinstance(dep, str) for dep in deps):
            report(f"invalid declared skills: {name}")
            continue
        required.update(deps)
        for dep in deps:
            if dep not in installed and dep not in adapters:
                report(f"missing required skill: {name} -> {dep}")

    for alias, adapter in adapters.items():
        target = adapter.get("skill")
        if target not in installed:
            report(f"missing adapter skill: {alias} -> {target}")
        elif alias not in texts.get(target, "") or not adapter.get("reason"):
            report(f"adapter must document alias and reason: {alias}")
        if alias in installed or alias not in required:
            report(f"stale skill adapter: {alias}")

    for path in sorted((root / ".agents").rglob("*.md")):
        rel = path.relative_to(root).as_posix()
        name = path.relative_to(root).parts[2] if rel.startswith(".agents/skills/") else ""
        for ref, bundled in resources(path.read_text(encoding="utf-8")):
            if ref.startswith(("#", "//")) or re.match(r"^[a-zA-Z][\w+.-]*:", ref) or placeholder(ref):
                continue
            target = unquote(ref.split("#", 1)[0].split("?", 1)[0]).strip("<>")
            candidate = path.parent / target
            if not inside(candidate):
                report(f"resource escapes repository: {rel}: {ref}")
                continue
            if candidate.exists():
                continue
            # Inline paths in local procedures may be repo-relative. Markdown
            # links are always relative to their containing document.
            if bundled and skills.get(name, {}).get("owner") == "local" and (root / target).exists():
                continue
            if not exempt(name, "resource", rel, ref):
                report(f"missing resource: {rel}: {ref}")

    for command in commands:
        name, text = command["skill"], command["text"]
        if text not in texts.get(name, ""):
            report(f"stale command declaration: {name}: {text}")
        cwd = root / command["cwd"]
        target = command["target"]
        if not inside(cwd) or not inside(cwd / target):
            report(f"command escapes repository: {text}")
            continue
        if command["kind"] == "package":
            invocation = re.search(r"\b(?:npm|pnpm)\s+(?:run\s+)?([\w:.-]+)", text)
            if not invocation or invocation[1] != target:
                report(f"command target does not match declared text: {text}: {target}")
            try:
                package = json.loads((cwd / "package.json").read_text())
                if not package.get("scripts", {}).get(target):
                    report(f"missing package command: {text} ({command['cwd']})")
            except (OSError, ValueError) as exc:
                report(f"unreadable command package: {text}: {exc}")
        elif command["kind"] == "file":
            if target not in text.split():
                report(f"command target does not match declared text: {text}: {target}")
            if not (cwd / target).is_file():
                report(f"missing command file: {text}: {target}")
        else:
            report(f"unsupported command kind: {text}")

    for index, exc in enumerate(exceptions):
        if index not in used_exceptions:
            report(f"stale exception: {exc}")
    return errors


def main():
    try:
        errors = check(ROOT)
    except (OSError, ValueError, TypeError, KeyError, AttributeError) as exc:
        errors = [f"invalid skill input: {exc}"]
    if errors:
        for error in errors:
            print(f"[FAIL] skills: {error}", file=sys.stderr)
        return 1
    print("[PASS] skills: ownership, metadata, lock provenance, resources, commands, dependencies and budgets")
    return 0


if __name__ == "__main__":
    sys.exit(main())
