# Runtime Architecture

`dn-kernel` is split into a small CLI and a reusable runtime crate.

## Components

- `apps/dn-cli`
  - parses commands and flags
  - renders text, JSON, and markdown
  - owns exit-code behavior for CI and local review flows
- `crates/dn-runtime`
  - resolves and validates profiles
  - walks files with ignore-aware traversal
  - runs heuristic local rules plus a registry-backed multi-language deterministic rule layer
  - orchestrates optional worker and provider passes
  - emits schema v2 reports and structured diagnostics
- `crates/dn-ipc`
  - shared request/response protocol for workers
- `workers/python`
  - external Python worker implementation

## Data flow

1. CLI selects command, root, profile, output mode, and scan flags.
2. Runtime resolves the effective profile from builtin or local sources.
3. Runtime validates the profile and emits warnings/errors as diagnostics.
4. Scanner builds include/exclude selectors and walks files.
5. Each candidate file is:
   - filtered by include/exclude policy
   - filtered by text/binary policy
   - skipped when limits are exceeded
   - analyzed by deterministic rules
   - optionally sent to worker integrations when suspicious patterns match
   - optionally sent to provider integrations when enabled
6. Findings, counters, diagnostics, and integration usage are assembled into schema v2 output.

## Public contracts

The main public contracts are:

- CLI commands and flags
- exit-code behavior
- profile semantics
- JSON schema versioning

See `docs/compatibility.md` and `docs/output.md` for the formal compatibility surface.

## Extensibility points

- deterministic rules: local heuristics in `run_local_rules` plus registry-backed multi-language rules in `crates/dn-runtime/src/rules.rs`
- workers: `WorkerRegistry` and `WorkerSession`
- providers: `crates/dn-runtime/src/provider.rs`
- docs updates: `docs/protocol.md`, `docs/providers.md`, `docs/output.md`
