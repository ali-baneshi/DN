# Security Policy

## Supported versions

`dn-kernel` is currently pre-release. Security support is best effort; important issues are prioritized quickly when they affect trust boundaries, data exposure, or scan integrity.

## Reporting vulnerabilities

If you believe you found a security issue:

1. Avoid posting it publicly before triage.
2. Contact the maintainer through a private reporting channel.
3. Include:
   - version or commit SHA,
   - exact command/profile path,
   - reproduction steps,
   - whether untrusted repository inputs are required,
   - whether provider/worker integrations are involved.

If no dedicated contact is published, request a private reporting path from the maintainer before public disclosure.

## Expected risk areas

- `--content` previews can expose secrets or sensitive code.
- provider integrations may disclose code to configured endpoints depending on profile settings.
- worker integrations execute local tools and should be treated as trusted local extensions.
- scanning large or adversarial repositories can create CPU or memory pressure within configured limits.
- JSON and Markdown reports can contain file paths, previews, and finding metadata; treat them as potentially sensitive artifacts.

## Hardening guidance for operators

- Keep custom profiles under version control.
- Review profiles before enabling `ai`, `--python-worker`, or `--strict-integrations` in shared environments.
- Prefer `--json` in CI so `diagnostics` can be inspected programmatically.
- Avoid sharing `--content` reports outside controlled environments.
- Use explicit profiles rather than ad-hoc local file paths when repeatability matters.
