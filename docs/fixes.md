# Safe Auto-Fix

`dn-cli fix <path>` applies a narrow set of deterministic fixes that are intentionally low risk.

## Current fixable rules

- `todo-comment`
- `debug-print`

## Behavior

- `--dry-run` reports which files would change
- fixes only apply to findings with concrete line numbers
- non-fixable rules remain report-only
- the current implementation only removes whole-line TODO/debug-print leftovers; it does not rewrite expressions or control flow

## Safety posture

`--fix` is intentionally conservative. It is not a code formatter, codemod engine, or semantic refactoring system.
