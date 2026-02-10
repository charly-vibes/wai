## Context
The current phase order in the project state machine is research → design → plan → implement → review → archive. The design-practice skill defines a workflow that defers tactical planning until after a design direction is chosen, which implies design should occur before planning execution details. Aligning the state machine with the agent workflow avoids conflicting guidance.

## Goals / Non-Goals
- Goals:
  - Reorder phases so design precedes planning.
  - Clarify what artifacts belong in the plan phase versus the design phase.
- Non-Goals:
  - Introduce new phases beyond the existing six.
  - Enforce phase transitions or validation rules.

## Decisions
- Decision: Swap the order of plan and design.
- Rationale: The design-practice workflow (direction → design → develop) implies tactical planning should occur after design is selected. The state machine should represent the sequence agents follow to keep phase guidance consistent.

## Risks / Trade-offs
- Risk: Existing projects already in the plan or design phase may have expectations about ordering.
  - Mitigation: Keep transitions flexible and document the updated semantics.

## Migration Plan
- Update spec and CLI phase ordering.
- Keep existing project states intact; users can move phases manually if desired.

## Open Questions
- Should the CLI surface a message when a project is in an older order? (defer unless requested)
