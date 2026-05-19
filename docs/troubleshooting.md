# Troubleshooting

## Unknown profile

Use a known profile name:

```bash
cargo run -p dn-cli -- scan . --profile quick
```

If using a local profile, ensure the file is under `<root>/.dn/profiles/<name>.toml|yml|yaml`.

## Worker appears unused

Worker analysis only runs for suspicious files and supported languages.
Use `--json` and inspect `integrations.worker` and `diagnostics`.

## Provider issues

If provider review is enabled but appears absent:

- inspect `integrations.provider`
- inspect `diagnostics`
- verify profile `ai` settings
- check whether suspicious patterns matched at all

## `--hidden` appears ineffective

- confirm the intended root path
- check `.gitignore` and profile `exclude_globs`
- verify that the profile itself does not override hidden behavior unexpectedly

## JSON output looks incomplete

- use `--json` explicitly
- use `--content` if previews are needed
- avoid `--summary-only` if you want full `files` entries

## `--fail-on` behavior surprises me

- exit code `2` means scan succeeded but findings crossed the configured threshold
- exit code `1` means scan or configuration failed
- exit code `3` is reserved for `doctor` and `validate-profile`

## `--content` leaks sensitive text

- avoid sharing reports generated with `--content`
- prefer `--summary-only --json` in CI when public artifacts are involved
- rotate secrets if sensitive material was exposed in logs
