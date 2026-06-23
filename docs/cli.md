# CLI Guide

<<<<<<< HEAD
`dn-cli` exposes a dual-mode surface: local review and CI-friendly gating.
=======
`dn-cli` exposes two equivalent entry points:

- `scan` (preferred): run a review and emit output.
- `review`: explicit alias of `scan` with identical options.
>>>>>>> feature/persistent-workers

## Commands

### `scan <path>`

<<<<<<< HEAD
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
=======
Scan the repository/folder at `<path>`.

Examples:

```bash
dn-cli scan . --profile quick
dn-cli scan . --profile security --json
dn-cli scan . --profile architecture --markdown
dn-cli scan . --profile quick --hidden
dn-cli scan . --profile my-security
```

### `review <path>`

Equivalent to `scan`.

Examples:

```bash
dn-cli review . --profile quick
dn-cli review . --profile architecture --json
```

## Global behavior notes

- `--profile` can be a built-in profile name, a local profile name, or a direct file path.
- If the profile is unknown, scan exits with an error and a list of available profiles.
- Only one output mode is allowed: `--json` or `--markdown`.
- `--help` for either subcommand shows the same flag set.

## Flags

- `--profile <name|path>`
  - Built-in profile name (`quick`, `security`, `architecture`, ...), local profile name (`my-security`), or explicit file path.
  - Local names are resolved from `<scan root>/.dn/profiles/<name>.toml|.yml|.yaml`.
  - `<scan root>` is the `path` argument, such as `.` in `dn-cli scan . --profile quick`.
- `--json`
  - Emit machine-readable JSON report.
  - Conflicts with `--markdown` and is optional.
- `--markdown`
  - Emit structured Markdown report for PRs and review notes.
  - Conflicts with `--json` and is optional.
- `--content`
  - Include short `content_preview` for each scanned file in JSON/Markdown output.
  - `content_preview` can expose secrets; avoid posting raw previews in public logs.
- `--hidden`
  - Include dotfiles and dot-directories in discovery.
- `--python-worker`
  - Force Python worker enablement (if supported by the scan context).
  - Requires Python-capable runtime when using the CLI image.

## Docker usage

```bash
# Build locally first:
# docker build -t dn-kernel -f docker/Dockerfile .
docker run --rm -v "$PWD":/workspace -w /workspace dn-kernel \
  scan /workspace --profile quick --json
```

This local image runs `dn-cli` as the entrypoint, so any CLI arguments can be passed after the image name.

Docker readiness note:
this repository's container configuration is reviewed, but container validation was not completed in this environment
because external registry/network TLS timeouts prevented reliable image tooling access. The limitation is environmental,
not a confirmed functional defect in the CLI/runtime code.

The default image is CLI-only; Python worker support requires a custom image with Python available.

## Error behavior

User configuration errors should never panic. Expected failures (unknown profile,
malformed profile, worker/protocol errors) are reported as plain CLI errors and
non-zero exit codes.

Diagnostics from runtime execution are also printed under `errors` in structured
outputs (`--json`/`--markdown`) so failures can be consumed in automation.
>>>>>>> feature/persistent-workers
