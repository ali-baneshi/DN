# Roadmap

<<<<<<< HEAD
`dn-kernel` `v1.0.0` is released; the roadmap now focuses on post-1.0 hardening and expansion.

## Current direction (implemented)

- Schema v2 report contract with structured diagnostics
- Dual-mode CLI for local review and CI quality gates
- Built-in and local profile loading with validation and inheritance
- Optional Python worker integration
- Optional provider integration with explicit strict/non-strict behavior
- Community health files and release guidance for GitHub-first open-source maintenance

## Near-term milestones

1. **Release packaging and versioning**
   - add reproducible source release process
   - define release tags/version bump policy
   - optionally add binary artifacts in a later phase
2. **Provider reliability**
   - expand `ollama` failure coverage and UX
   - add more provider health checks in `doctor`
3. **Operational polish**
   - richer diagnostics grouping and filtering
   - broaden runtime tests for integration edge cases
4. **Security and maintenance**
   - document secure usage patterns for untrusted repositories
   - add more explicit support boundaries and maintainer expectations
=======
`dn-kernel` is pre-release and intentionally scoped for practical use.

## Current direction (implemented)

- Hardened CLI profile handling:
  - built-in + local profile loading
  - inheritance and validation
  - non-panicking user error handling
- Stable scan counters and reporting semantics
- Optional Python worker integration
- Profile- and output-mode-aware markdown/json reporting
- Documentation set for CLI, profiles, provider model, and protocol

## Near-term milestones

1. **Release packaging and onboarding**
   - document reproducible install artifacts
   - add release checks and versioned changelog expectations
2. **Provider reliability**
   - harden and document Ollama/local provider failure modes
   - add explicit fallback strategy and opt-in strictness flags
3. **Operational polish**
   - richer configuration validation diagnostics
   - optional threshold-based non-zero exit for findings
4. **Security and maintenance**
   - add security testing checklist (dependency pinning, content leakage guidance)
   - improve scan diagnostics grouping by category/source
>>>>>>> feature/persistent-workers

## Out of scope for this release

- No UI/daemon architecture is planned in this stage.
<<<<<<< HEAD
- No cross-platform binary publishing in this phase.
- No breaking schema changes beyond the documented move to report schema v2.
=======
- No behavior changes that alter default include/exclude policy without explicit profile/flag change.
- No breaking report schema changes without migration notes.
>>>>>>> feature/persistent-workers
