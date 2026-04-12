# Phase 16: Semantic Constraint Logic - Context

**Gathered:** 2026-03-20
**Status:** Historical planning context; implementation completed with the current field names noted below

<domain>
## Phase Boundary

Add policy-aware parsing so modern templates can declare constraints, the parser can detect impossible states during parse time, violations can be surfaced clearly, and `--strict-policy` can turn those violations into parse failure behavior. This phase clarifies how constraint-driven validation behaves; it does not add new policy systems beyond the roadmap scope.

</domain>

<decisions>
## Implementation Decisions

### Constraint authoring
- Use a hybrid authoring model: simple constraints live with each field, while the design should leave room for a separate section for more advanced cases later.
- Ship only the roadmap constraint types in this phase: `min`, `max`, `choices`, and `regex`.
- Keep the authoring model future-friendly so additional constraint kinds can slot in later without redesigning the template shape.
- Constraints should be profile-based rather than purely ad hoc; authors should be able to opt into stricter policy behavior instead of relying only on unconstrained fields.

### Violation reporting
- In non-strict mode, policy violations should surface in both normal warnings and rich tracing events.
- Each reported violation should include rich context: field, bad value, failed rule, and record or location context.
- Default human-facing presentation should emphasize summarized counts plus representative samples rather than dumping every violation.
- Policy violations should fit the existing warning/reporting model in the clearest way possible while still standing out clearly.

### Strict policy mode
- `--strict-policy` should be a policy-only gate; declared constraint violations fail the command, but ordinary non-policy warnings should remain warnings.
- Mixed valid/invalid results should default to no normal parsed data output when strict policy fails.
- Strict-mode behavior can vary by input source if needed, but the overall experience should stay predictable.
- The strict-mode experience should balance CI-friendliness with operator clarity.

### Constraint semantics
- Missing constrained fields should be treated as rule-dependent rather than always valid or always invalid.
- If type conversion fails before a numeric check, user-facing behavior should capture both the conversion problem and the resulting constraint failure.
- `regex` constraints should pass on any regex match by default.
- When one field violates multiple constraints, reporting should show a concise headline failure while retaining full detail in tracing or deeper reporting.

### Claude's Discretion
- Exact evaluation model when a field has multiple constraints, as long as it remains clear to authors.
- Exact separation or categorization of policy violations within the existing warning model.
- The specific input-source differences, if any, for `--strict-policy` behavior.

</decisions>

<specifics>
## Specific Ideas

- Non-strict runs should feel actionable rather than noisy: summarized policy failures with representative examples, plus richer trace detail underneath.
- Strict mode should behave well in both CI and human troubleshooting workflows.
- The template format should feel ready for future constraint growth without expanding this phase's shipped scope.

</specifics>

<deferred>
## Deferred Ideas

None - discussion stayed within phase scope.

</deferred>

---

*Phase: 16-semantic-constraint-logic*
*Context gathered: 2026-03-20*
