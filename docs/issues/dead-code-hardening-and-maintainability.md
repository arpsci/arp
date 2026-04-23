# Issue Draft: Dead Code Cleanup + Hardening Pass (No Behavior Change)

## Summary
This issue proposes a focused hardening pass to keep current functionality intact while removing dead/unused code paths, reducing panic risk, and making the codebase safer for upcoming feature work.

This is intentionally **not** a feature issue. It is a reliability and maintainability baseline issue that should reduce regressions and improve confidence for future platform evolution.

## Why now
Recent lint/build inspection shows avoidable risk in core paths:
- Dead-code suppression in conversation history (`#[allow(dead_code)]`).
- Redundant locals and duplicated logic in nodes panel render path.
- Multiple panic-prone `unwrap()` calls across UI/runtime/event paths.
- Large functions with mixed responsibilities that increase bug surface.
- Missing tests around critical flows (event ledger, sidecar dispatch, queue drain semantics).

## Scope
1. Remove confirmed dead code and dead-code suppressions.
2. Refactor obvious no-op/redundant patterns (shadowed locals, identical `if` branches).
3. Replace panic-prone `unwrap()` where recoverable handling is possible, especially in UI/event-loop paths.
4. Split oversized functions into smaller internal helpers without changing external behavior.
5. Add targeted regression tests to lock current behavior.
6. Add CI quality gate for dead/unused warnings and compile checks.

## Non-goals
- No product behavior changes.
- No protocol/schema redesign.
- No UI redesign.
- No runtime architecture rewrite.

## Proposed implementation plan
1. **Dead code cleanup**
- Remove unused fields/arguments and eliminate `#[allow(dead_code)]` where not needed.
- Keep serialized/public structures backward-compatible unless explicitly versioned.

2. **Panic surface reduction**
- Replace lock/result `unwrap()` in non-critical paths with graceful handling and error logging.
- Keep fail-fast behavior only where invariants are truly unrecoverable and documented.

3. **Complexity reduction**
- Break large render/loop functions into private helpers by responsibility:
  - queue intake
  - evaluator dispatch
  - researcher dispatch
  - HTTP emission
- Keep signatures stable for public APIs.

4. **Test coverage additions**
- Add tests for:
  - conversation event queue de-dup semantics
  - evaluator/researcher queue ordering
  - event ledger append/read invariants
  - metrics event serialization invariants

5. **Quality gates**
- CI step: `cargo check --all-targets`
- CI step: `cargo clippy --all-targets --all-features -- -W dead_code -W unused`
- Optional staged stricter gate for `clippy::unwrap_used` on selected modules.

## Acceptance criteria
- Project builds cleanly with `cargo check --all-targets`.
- No `dead_code` or `unused` warnings in touched modules.
- No behavior regressions in manual smoke flow (start conversation, evaluator/researcher sidecars, HTTP send path).
- New tests pass locally and in CI.
- Functionality and outputs remain equivalent for existing workflows.

## Initial candidate hotspots
- `src/agents/agent_conversation_loop.rs`
- `src/ui/nodes_panel/render.rs`
- `src/ui/nodes_panel/body.rs`
- `src/ui/python_panel.rs`
- `src/run/event_ledger.rs`
- `src/metrics/mod.rs`

## Suggested labels
- `enhancement`
- `tech-debt`
- `stability`
- `refactor`

## Notes for maintainers
This issue is designed to complement existing architecture/feature issues by creating a safer baseline first. It should be executable in small PRs with strict no-behavior-change review criteria.
