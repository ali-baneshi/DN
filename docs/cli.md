# CLI Guide

`dn-cli` exposes a dual-mode surface: local review and CI-friendly gating.

## Commands

### `scan <path>`

Run a repository scan.

### `review <path>`

Alias of `scan` with the same flags.

### `profiles list <root>`

List built-in and local profiles visible from the given scan root.

### `profiles show <name-or-path> <root>`

Render the effective merged profile and any validation diagnostics.

### `validate-profile <path> <root>`

Validate and resolve a profile file without scanning repository content.

### `doctor <root>`

Run lightweight environment checks for local profile presence, worker scripts, runtimes, and example profile availability.

### `fix <path>`

Apply safe automatic fixes for a narrow subset of deterministic rules, including low-risk cleanup and explicit review markers for wildcard imports.

### `rules`

List the built-in deterministic rule registry and whether each rule supports `--fix`.

## Scan flags

- `--profile <name|path>`: built-in profile, local profile, or explicit file path
- `--json`: emit schema v2 JSON
- `--markdown`: emit markdown report
- `--content`: include bounded `content_preview`
- `--hidden`: include hidden files and directories
- `--python-worker`: enable Python worker in addition to profile settings
- `--fail-on <none|info|low|medium|high|critical>`: return exit code `2` when threshold is reached
- `--summary-only`: keep summary/stats/diagnostics but emit an empty `files` array in JSON
- `--strict-integrations`: convert provider/worker failures from diagnostics into hard failures
- `--max-files <n>`: override profile file limit for this run
- `fix --dry-run`: preview safe autofix candidates without mutating files

## Exit codes

- `0`: success, no threshold trip
- `1`: scan/runtime/configuration failure
- `2`: scan succeeded and threshold was tripped
- `3`: doctor/validation failure

## Examples

```bash
dn-cli scan . --profile quick
dn-cli scan . --profile security --json --fail-on medium
dn-cli review . --profile architecture --markdown --content
dn-cli scan . --profile quick --summary-only --json
dn-cli profiles list . --json
dn-cli profiles show quick . --json
dn-cli profiles show maintainer-review . --json
dn-cli validate-profile examples/profiles/my-security.toml . --json
dn-cli doctor . --json
dn-cli rules --json
dn-cli fix . --profile quick --dry-run --json
```
