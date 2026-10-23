# ADR-0033: Phase 0 Forensic Inventory, Correctness Audit, and Non-Production-Ready Determination

## Status
Accepted

## Context
Following previous development iterations and an audit that declared the system "100% Production Ready", a rigorous forensic re-assessment was initiated. The mandate requires an unsparing inventory of every workspace crate, executable, JNI/FFI binding, shader, GIS dataset, compiler, streaming system, physics model, gameplay workflow, test, benchmark, and CI workflow.

The audit sought to identify:
1. Synthetic/generated geography disguised as surveyed data.
2. Stubbed, disconnected, or no-op functions in production code paths.
3. Tests that validate internal stub assumptions rather than real end-to-end behavior.
4. Premature claims of production readiness.

## Decisions

1. **Classify Platform Status Honestly**:
   - Classify all requirements across Stages 0 through 11 with explicit audit classifications: `VERIFIED`, `LOW-QUALITY`, `PARTIAL`, `BROKEN`, `MISSING`, `BLOCKED`, and `UNTESTED` with severities `P0` through `P3`.
   - Maintain this classification in `docs/FeatureQualityMatrix.md` and machine-readable `docs/FeatureQualityMatrix.json`.

2. **Revoke Production Readiness Ratification**:
   - Formally declare the Real Earth World Platform as **NOT PRODUCTION READY**.
   - Block production status pending remediation of 3 critical P0 blockers:
     - Synthetic geography across 32 catalog cities.
     - Android mobile JNI surface rendering stub and `FlatGround` reliance.
     - Live multiplayer networking disconnection from `world-viewer`.

3. **Identify Test Suite Internal Assumption Bias**:
   - Explicitly document tests that assert on internal stub outputs (such as `assert_eq!(nativeSurfaceCreated(...), 0)`) rather than real graphics presentation.
   - Mandate that subsequent test suites validate observable physical and graphical side-effects.

4. **Preserve Append-Only History**:
   - Maintain strict append-only discipline for `ImplementationPlan.md`, `Todo.md`, and `Changelog.md`, recording the Phase 0 audit findings and remediation ledger without erasing prior history.

## Consequences

* **Positive**: Full engineering honesty is restored. Stakeholders have an exact, verified inventory of what works (San Francisco & Bengaluru desktop simulation) and what requires real engineering work.
* **Accountability**: Elimination of fake certification prevents shipping non-functional mobile surfaces or synthetic cities represented as surveyed GIS truth.
* **Negative**: Requires substantial GIS acquisition and mobile graphics implementation before a commercial Version 1.0.0 can be legitimately certified.
