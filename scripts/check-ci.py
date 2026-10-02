#!/usr/bin/env python3
"""Offline CI contract, deliberately scoped to ci.yml's explicit YAML layout.

No YAML dependency: accept only the simple job/needs/filter forms this workflow
uses, reject missing/unsupported forms. Path fixtures are independent expected
coverage, not derived from the workflow being checked.
"""
import ast
import fnmatch
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
EXEMPT = {'ci-success', 'pr-readiness'}  # Readiness is independently required.
DIRECT = {'architecture': ['bash scripts/check-architecture.sh', 'python3 scripts/check-ci.py',
                           'python3 scripts/test-ci.py', 'python3 scripts/test-sensors.py',
                           'python3 scripts/check-skills.py', 'python3 scripts/test-skills.py',
                           'python3 scripts/test-build.py'],
          'rust': ['bash scripts/test-wasm.sh'],
          'security': ['cargo audit --file Cargo.lock'],
          'security-npm': ['bash scripts/npm-audit.sh'],
          'wasm': ['pnpm run build:wasm', 'pnpm run check-size'],
          'web': ['python3 scripts/wasm-freshness.py --check', 'pnpm exec tsc --noEmit',
                  'node --test scripts/test-playwright-config.mjs'],
          'e2e': ['python3 scripts/wasm-freshness.py --check', 'pnpm run build:web',
                  'PRODUCTION_E2E: 1']}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def jobs_in(text):
    require('\njobs:\n' in text, 'missing jobs mapping')
    text = text.split('\njobs:\n', 1)[1]
    matches = list(re.finditer(r'^  ([a-z][a-z0-9-]*):\s*$', text, re.M))
    require(bool(matches), 'no jobs found / unsupported workflow layout')
    require(len({match[1] for match in matches}) == len(matches), 'duplicate job keys')
    return {match[1]: text[match.end():matches[index + 1].start() if index + 1 < len(matches) else len(text)]
            for index, match in enumerate(matches)}


def filters_in(changes):
    filters = {}
    current = None
    for line in changes.splitlines():
        if match := re.fullmatch(r'            ([a-z]+):', line):
            current = match[1]
            filters[current] = []
        elif match := re.fullmatch(r"              - '([^']+)'", line):
            require(current is not None, 'path before filter category')
            filters[current].append(match[1])
    require(set(filters) == {'rust', 'web', 'product', 'harness'}, 'missing/unknown filter categories')
    require(all(filters.values()), 'empty filter category')
    return filters


def needs_in(block):
    match = re.search(r'^    needs: \[([^\]]+)\]\s*$', block, re.M)
    require(match is not None, 'missing/unsupported needs list')
    names = [name.strip() for name in match[1].split(',')]
    require(len(names) == len(set(names)), 'duplicate needs entries')
    return set(names)


def evaluate(condition, outputs, results, event='pull_request'):
    """Evaluate only the boolean GitHub-expression subset used by our filters."""
    condition = re.sub(r"github.event_name != 'pull_request'", str(event != 'pull_request'), condition)
    condition = condition.replace('always()', 'True')
    condition = re.sub(r"needs.changes.outputs.([a-z]+) == 'true'",
                       lambda match: str(outputs[match[1]]), condition)
    condition = re.sub(r"needs.([a-z0-9-]+).result == 'success'",
                       lambda match: str(results[match[1]]), condition)
    condition = condition.replace('||', ' or ').replace('&&', ' and ')
    condition = re.sub(r'\btrue\b', 'True', condition)
    condition = re.sub(r'\bfalse\b', 'False', condition)
    try:
        tree = ast.parse(' '.join(condition.split()), mode='eval')
    except SyntaxError as error:
        raise ValueError(f'unsupported applicability expression: {condition}') from error

    def boolean(node):
        if isinstance(node, ast.Constant) and type(node.value) is bool:
            return node.value
        if isinstance(node, ast.BoolOp) and isinstance(node.op, (ast.And, ast.Or)):
            values = [boolean(value) for value in node.values]
            return all(values) if isinstance(node.op, ast.And) else any(values)
        raise ValueError(f'unsupported applicability expression: {condition}')
    return boolean(tree.body)


def package_manager_version(root_package, web_package):
    manager = root_package.get('packageManager', '')
    require(manager == web_package.get('packageManager'), 'root/web packageManager pins differ')
    match = re.fullmatch(r'pnpm@(\d+\.\d+\.\d+)', manager)
    require(match is not None, 'root/web need the same exact pnpm packageManager version')
    return match[1]


def lockfile_committed(text, is_tracked=None):
    """A workflow that forbids fresh resolves needs a committed Cargo.lock."""
    if '--locked' not in text and 'cargo audit --file Cargo.lock' not in text:
        return
    if is_tracked is None:
        def is_tracked():
            probe = subprocess.run(['git', '-C', str(ROOT), 'ls-files', '--error-unmatch', 'Cargo.lock'],
                                   capture_output=True)
            return probe.returncode == 0
    require(is_tracked(), 'Cargo.lock is used by --locked/audit steps but is not committed '
                          '(un-ignore and commit it, or drop --locked)')


def check(text):
    manager = package_manager_version(json.loads((ROOT / 'package.json').read_text()),
                                      json.loads((ROOT / 'web/package.json').read_text()))
    jobs = jobs_in(text)
    for name, block in jobs.items():
        if 'uses: pnpm/action-setup@' in block:
            versions = re.findall(r'uses: pnpm/action-setup@[^\n]+\n +with:\n +version: ([^\n]+)', block)
            require(len(versions) == block.count('uses: pnpm/action-setup@') and
                    all(version == manager for version in versions),
                    f'{name}: pnpm setup must match packageManager ({manager})')
    require(EXEMPT | {'changes'} <= jobs.keys(), 'required coordination job missing')
    expected = set(jobs) - EXEMPT
    aggregator = jobs['ci-success']
    needs = needs_in(aggregator)
    require(needs == expected, f'jobs/ci-success.needs mismatch: {needs ^ expected}')
    env_pairs = re.findall(r'^          ([A-Z_0-9]+): \$\{\{ needs\.([a-z0-9-]+)\.result \}\}', aggregator, re.M)
    env = dict(env_pairs)
    require(len(env) == len(env_pairs) == len(expected), 'duplicate/missing result env entries')
    require(set(env.values()) == expected, 'result env does not cover all jobs')
    pairs = re.findall(r'\[([a-z0-9-]+)\]="\$([A-Z_0-9]+)"', aggregator)
    require(len(pairs) == len(expected) and {job for job, _ in pairs} == expected,
            'shell results map does not cover all jobs')
    require(all(env.get(variable) == job for job, variable in pairs), 'shell results are miswired')
    filters = filters_in(jobs['changes'])
    conditions = {}
    for name in expected - {'changes'}:
        deps = needs_in(jobs[name])
        require(deps <= jobs.keys() and 'changes' in deps, f'{name}: invalid dependencies')
        condition = re.search(r'^    if: >\n((?:      .*\n)+)', jobs[name], re.M)
        require(condition is not None, f'{name}: unsupported/missing applicability condition')
        categories = set(re.findall(r"needs.changes.outputs.([a-z]+) == 'true'", condition[1]))
        require(bool(categories) and categories <= filters.keys(), f'{name}: invalid filters')
        conditions[name] = condition[1]
    fixtures = json.loads((ROOT / 'scripts/fixtures/ci-paths.json').read_text())
    for path, required in fixtures.items():
        triggered = {category for category, patterns in filters.items()
                     if any(fnmatch.fnmatchcase(path, pattern) for pattern in patterns)}
        outputs = {name: name in triggered for name in filters}
        results = {'changes': True}
        visiting = set()

        def applies(name):
            if name not in results:
                require(name not in visiting, 'cyclic job dependencies')
                visiting.add(name)
                dependencies = {dep: applies(dep) for dep in needs_in(jobs[name])}
                results[name] = evaluate(conditions[name], outputs, dependencies)
                visiting.remove(name)
            return results[name]

        active = {name for name in conditions if applies(name)}
        missing = set(required) - active
        require(not missing, f'{path}: required jobs skipped: {sorted(missing)}')
        for name in active:
            require(needs_in(jobs[name]) - {'changes'} <= active,
                    f'{path}: {name} depends on a skipped job')
    for name, condition in conditions.items():
        require(evaluate(condition, dict.fromkeys(filters, False), dict.fromkeys(jobs, True), 'push'),
                f'{name}: non-PR verification must run regardless of filters')
    for job, commands in DIRECT.items():
        require(job in jobs, f'missing sensor job: {job}')
        executable = '\n'.join(line for line in jobs[job].splitlines() if not line.lstrip().startswith('#'))
        for command in commands:
            require(command in executable, f'{job}: missing direct sensor {command}')
    require('cargo generate-lockfile' not in text, 'audit must inspect committed Cargo.lock')
    lockfile_committed(text)
    require('include-hidden-files: true' in jobs['wasm'], 'WASM provenance must travel with artifact')
    return len(expected), len(fixtures)


def main():
    try:
        count, paths = check((ROOT / '.github/workflows/ci.yml').read_text())
        print(f'[PASS] CI: {count} jobs == needs == result map; {paths} applicability fixtures; direct sensors')
    except (OSError, ValueError, KeyError) as error:
        print(f'[FAIL] CI coherence: {error}', file=sys.stderr)
        print('FIX: keep path filters, job conditions, needs and result maps coherent.', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
