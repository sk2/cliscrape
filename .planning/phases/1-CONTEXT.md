# Phase 1 Context: Core Parsing Engine

## Implementation Decisions

### Regex Library (Simple Start)
- **Built-in Only:** Phase 1 will use a hardcoded internal library of common regex macros (e.g., `{{ipv4}}`, `{{mac_address}}`).
- **No Nesting:** Macros will not support nesting in this initial phase to keep the regex compilation logic simple.
- **Local Shadowing:** If a template defines a value with the same name as a macro, the template definition wins.

### Error & Fallback Behavior (Standard FSM)
- **Unmatched Lines:** If no rules match a line in the current state, the line is discarded and the machine stays in the current state.
- **Required Values:** If a `Record` action is triggered but a value marked as `Required` is empty, the record is silently discarded.
- **Invalid Transitions:** The engine will error and exit if a rule attempts to transition to a non-existent state.
- **Verbosity:** The engine remains silent, outputting only the resulting data or hard errors.

### Data Handling
- **String Primacy:** All captured data will be stored and returned as strings. No type coercion (Integer, IP, etc.) will occur in this phase.
- **Internal Storage:** Records are stored as a `Vec<HashMap<String, String>>`.

## Deferred to Future Phases
- External user-defined macros (`macros.yaml`).
- Nested macro definitions.
- Automatic type coercion (Phase 3).
- Global/Fallback states.

---
*Created: 2024-05-22*
