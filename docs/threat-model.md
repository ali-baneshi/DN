# Threat Model

## Trusted and untrusted inputs

Untrusted inputs:

- scanned repository content
- local profile files from repositories under review
- worker responses
- provider responses

Trusted inputs:

- built-in profile definitions in the source tree
- local maintainer-controlled configuration and release process

## Key boundaries

- scan traversal does not follow symlinks
- profile name/path validation rejects traversal patterns
- inheritance depth is bounded
- worker/provider failures are surfaced as diagnostics
- provider and worker outputs are normalized before becoming findings
- `--content` is opt-in because it can leak sensitive material into reports

## Risk areas

- scanning adversarial repositories can still consume CPU/time within configured limits
- profile files from untrusted repositories should be reviewed before enabling integrations
- provider-backed review can disclose code to a configured endpoint depending on provider settings
