# Change: Align project phase order with design-practice workflow

## Why
The current project phase order places planning before design, which conflicts with the design-practice workflow used by agents. The project state machine should mirror the describe→diagnose→delimit→direction→design→develop progression so phase guidance and artifacts align with how agentic workflows actually unfold.

## What Changes
- Reorder project phases to place design before planning.
- Clarify that the planning phase focuses on implementation planning after design is selected.
- Update documentation and state machine definitions to reflect the new order.

## Impact
- Affected specs: project-state-machine
- Affected code: `src/state.rs`, CLI phase ordering, any phase-related messaging
