# Compatibility Policy

`dn-kernel` `v1.0.0` treats public behavior as a stable compatibility surface for normal adoption.

## Compatibility surfaces

The main compatibility surfaces are:

- CLI command names and flag semantics
- JSON schema versioning
- exit code contract
- profile file semantics

## JSON schema policy

Current schema version: `2`

Rules:

- additive changes may happen within the same schema version
- breaking JSON shape changes require a schema version bump
- docs must be updated in `docs/output.md` and `README.md` whenever schema changes

## Exit codes

- `0`: success
- `1`: runtime/config failure
- `2`: threshold reached with `--fail-on`
- `3`: doctor/validate-profile failure

## Stable change policy

After `v1.0.0`, CLI/schema changes must:

- be documented in `CHANGELOG.md`
- update tests in the same patch
- update docs in the same patch
