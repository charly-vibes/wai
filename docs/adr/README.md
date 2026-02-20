# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) for the wai project.

## What is an ADR?

An Architecture Decision Record (ADR) is a document that captures an important architectural decision made along with its context and consequences.

## ADR Format

Each ADR follows this structure:

- **Title**: Short, descriptive name
- **Status**: Proposed, Accepted, Deprecated, Superseded
- **Context**: The situation and forces at play
- **Decision**: The choice that was made
- **Consequences**: The resulting context, including positive and negative effects

## Naming Convention

ADRs are numbered sequentially: `NNNN-title-with-dashes.md`

Example: `0001-separate-way-command-for-repository-checks.md`

## When to Create an ADR

Create an ADR when making decisions that:
- Affect the public API or command structure
- Change fundamental architecture or design patterns
- Have significant long-term consequences
- Involve trade-offs between competing concerns
- Need to be understood by future contributors

## ADR List

- [ADR 0001: Separate `wai way` Command for Repository Best Practices](0001-separate-way-command-for-repository-checks.md)

## References

- [Architecture Decision Records](https://adr.github.io/) - General ADR information
