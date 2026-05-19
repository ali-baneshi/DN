# Troubleshooting

## Unknown profile

Use a known profile name:

```bash
cargo run -p dn-cli -- scan . --profile quick
```

If using a local profile, ensure the file is under `<root>/.dn/profiles/<name>.toml|yml|yaml`.

If you are building a new custom profile, start from `examples/profiles/` and validate it before scanning:

```bash
dn-cli validate-profile examples/profiles/ci-fast.toml .
```

## Worker appears unused

Worker analysis only runs for suspicious files and supported languages.
Use `--json` and inspect `integrations.worker` and `diagnostics`.

## Provider issues

If provider review is enabled but appears absent:

- inspect `integrations.provider`
- inspect `diagnostics`
- verify profile `ai` settings
- check whether suspicious patterns matched at all

For `ollama`, only local endpoints are accepted right now. Remote URLs are rejected on purpose.

## Too many false positives

- reduce `rules.suspicious_patterns` in your profile if worker/provider analysis is triggering too broadly
- if provider review should use a different trigger set, define `[ai].suspicious_patterns` separately from `[rules].suspicious_patterns`
- prefer `pre-merge` or a trimmed custom profile in CI rather than the broadest local profile
- remember that obvious placeholders like `example`, `changeme`, and `${TOKEN}` are already suppressed by local secret-like rules

## Missed suspicious files

- add repository-specific terms to `rules.suspicious_patterns`
- if provider review needs broader coverage than workers, add `[ai].suspicious_patterns`
- use a profile that enables the worker and/or provider paths
- increase `limits.max_file_read_bytes` if important indicators are located after the initial preview window

## `--hidden` appears ineffective

- confirm the intended root path
- check `.gitignore` and profile `exclude_globs`
- verify that the profile itself does not override hidden behavior unexpectedly

## JSON output looks incomplete

- use `--json` explicitly
- use `--content` if previews are needed
- avoid `--summary-only` if you want full `files` entries

## `doctor` reports warnings

- `doctor` is intentionally advisory for missing local profiles, missing example profiles, or missing Python runtimes
- use it to understand environment readiness before enabling worker-backed profiles in CI or local review

## `--fail-on` behavior surprises me

- exit code `2` means scan succeeded but findings crossed the configured threshold
- exit code `1` means scan or configuration failed
- exit code `3` is reserved for `doctor` and `validate-profile`

## `--content` leaks sensitive text

- avoid sharing reports generated with `--content`
- prefer `--summary-only --json` in CI when public artifacts are involved
- rotate secrets if sensitive material was exposed in logs


## Java/TypeScript worker findings are missing

- confirm the file extension maps to a supported language such as `.java`, `.ts`, or `.js`
- ensure the active profile enables workers and includes suspicious patterns that match the file content
- inspect `integrations.worker.mode` and `diagnostics` in JSON output to confirm the worker actually ran
