# Troubleshooting

## Unknown profile

Use a known profile name:

```bash
cargo run -p dn-cli -- scan . --profile quick
```

<<<<<<< HEAD
If using a local profile, ensure the file is under `<root>/.dn/profiles/<name>.toml|yml|yaml`.

If you are building a new custom profile, start from `examples/profiles/` and validate it before scanning:

```bash
dn-cli validate-profile examples/profiles/ci-fast.toml .
```

## Worker appears unused

Worker analysis only runs for suspicious files and supported languages. Current practical language coverage includes Rust, Python, JavaScript, TypeScript, Java, Go, PHP, Ruby, and Shell/Bash across the core detector and workers where applicable.
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
=======
If using a local profile, ensure the file is under `<root>/.dn/profiles/<name>.toml|yml|yaml>`.

The error message includes available profile hints.

## Worker not used / silent

Worker checks are currently triggered only for suspicious files and supported file languages.
If no findings and no errors appear, it may mean no suspicious patterns matched.

Enable verbose diagnostics with `--json` and inspect `errors`.

## `--hidden` appears ineffective

- Ensure running from the intended root path.
- Confirm hidden path is not excluded by `.gitignore` plus profile `exclude_globs`.

## JSON/Markdown output looks incomplete

- Use `--json` or `--markdown` explicitly.
- Ensure `--content` is enabled if you need file previews.

## I get unknown profile errors

- The local profile directory for a scan is `<scan-root>/.dn/profiles`.
- If you want explicit file mode, pass an existing file path to `--profile`.
- When a profile cannot be resolved, help text now includes available profile hints.

## Untrusted profile files

- Review local profile content before use, especially when using profiles from shared repositories.
- Prefer explicit path mode only for trusted files under review.

## `--content` leaks sensitive text

- `--content` is intended for local review; output should remain local and access-limited.
- do not paste output that contains secrets, credentials, or tokens into public chat/dev logs.

## Docker validation blocked by network TLS

If build or run commands fail with registry/network TLS errors, this is commonly an environment issue (for example, restricted
egress or certificate interception) rather than a proven CLI/runtime defect.

Recommended response:

- Keep Docker instructions available for users.
- Record the exact `docker` error and network context in release notes.
- Continue validating via local CLI commands (`cargo run ...`) and smoke workflows.
>>>>>>> feature/persistent-workers
