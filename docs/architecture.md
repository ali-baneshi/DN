# Runtime Architecture

<<<<<<< HEAD
`dn-kernel` is split into a small CLI and a reusable runtime crate.
=======
`dn-kernel` is separated into a small CLI and a reusable runtime crate.
>>>>>>> feature/persistent-workers

## Components

- `apps/dn-cli`
  - parses commands and flags
<<<<<<< HEAD
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
- `workers/java`
  - Java-focused worker for higher-risk runtime/security patterns
- `workers/typescript`
  - JavaScript/TypeScript worker for web and service sink analysis

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
=======
  - loads profile suggestions for unknown-profile hints
  - renders report formats
- `crates/dn-runtime`
  - loads and merges profiles
  - walks files with ignore-aware traversal
  - runs deterministic rules
  - orchestrates optional worker and provider steps
  - generates structured report
- `crates/dn-ipc`
  - shared request/response data models for workers
- `workers/python`
  - external language worker implementation

## Data flow

1. CLI selects root + profile + output mode.
2. Runtime resolves profile (builtin / local file / explicit path) and applies inheritance.
3. Scanner builds include/exclude selectors and walks files.
4. Each candidate file is:
   - filtered by include/exclude
   - filtered out when not text/binary policy allows
   - skipped if too large or over global limits
   - read and analyzed by local rules
   - optionally analyzed by worker when suspicious patterns match
   - optionally analyzed by provider when enabled
5. Findings are merged, sorted, counted, and serialized into a `ScanReport`.

## Extensibility points

- Add deterministic checks in `run_local_rules`.
- Add new runtime workers via `WorkerRegistry`/`WorkerSession` and keep protocol compatibility with `crates/dn-ipc`.
- Add providers in `crates/dn-runtime/src/provider.rs` using `Provider::from_config`.
- Update docs (`docs/protocol.md`, `docs/providers.md`) alongside any protocol or provider changes.

## Extensibility checklist

- add parsing + validation tests for new config fields
- keep default behavior opt-out compatible
- keep worker/protocol failures non-fatal when possible
- surface integration mode and failures explicitly in `provider`/`worker` fields
- add integration coverage under `apps/dn-cli/tests`

The worker layer and provider layer remain independent, so workers can exist even when provider is disabled and vice versa.
>>>>>>> feature/persistent-workers
