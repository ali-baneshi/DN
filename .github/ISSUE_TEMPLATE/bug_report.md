---
name: Bug report
about: Report a reproducible bug
labels: bug
assignees: ''
---

## Summary

<<<<<<< HEAD
A concise statement of the observed behavior and expected behavior.
=======
A concise description of what happened.
>>>>>>> feature/persistent-workers

## Reproduction

```bash
<<<<<<< HEAD
# Replace with exact command
dn-cli scan <path> --profile <profile> [--json|--markdown] [--fail-on <severity>] [--summary-only]
=======
# Command

dn-cli scan ...
>>>>>>> feature/persistent-workers
```

## Environment

- OS:
<<<<<<< HEAD
- Shell:
- `rustc -V`:
- `cargo -V`:
- Profile name/path:
- Output mode:
- `--strict-integrations` used?:
- `schema_version` from JSON output (if applicable):

## Expected behavior

## Actual behavior

## Evidence

- Minimal sample repo/path or sanitized excerpt
- stdout/stderr
- JSON diagnostics payload or markdown report
- Relevant `.dn/profiles` content
=======
- Rust version (`rustc -V`):
- Command and args used:
- Profile:
- Output mode: (`default text` / `--json` / `--markdown`):

## Expected vs actual

What should happen:

What actually happened:

## Artifacts

If possible, attach:

- small sample repo/path
- command output (stdout/stderr)
- `.dn/profiles` content used (if custom)
>>>>>>> feature/persistent-workers
