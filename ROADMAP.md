# Roadmap

`dn-kernel` is pre-release and intentionally scoped for practical use.

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

## Out of scope for this release

- No UI/daemon architecture is planned in this stage.
- No cross-platform binary publishing in this phase.
- No breaking schema changes beyond the documented move to report schema v2.
