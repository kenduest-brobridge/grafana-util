# Architecture Review

Date: 2026-03-27
Scope: Rust runtime only
Audience: Maintainers
Status: Updated after follow-on `sync` and promotion work completed on 2026-03-27

## Purpose

This document records an architecture-focused review of the maintained Rust codebase.

It is intended to answer four questions:

- what the project is today
- where the current design is strong
- what looks risky or structurally weak
- what should be improved next

## Current Shape

The project is no longer just a collection of Grafana API wrappers.

The maintained Rust runtime already behaves like an operator-focused toolkit with five major domain surfaces:

- `dashboard`
- `datasource`
- `alert`
- `access`
- `sync`

At a high level, the current structure is:

- `rust/src/cli.rs`: unified entrypoint, namespaced command topology, and top-level help flow
- `rust/src/access/`: user, org, team, and service-account lifecycle workflows
- `rust/src/dashboard/`: export, import, diff, inspect, governance, screenshot, topology, and interactive browse/workbench flows
- `rust/src/datasource.rs` plus helper modules: datasource inventory, import/export/diff, and live mutation
- `rust/src/alert.rs` plus helper modules: alert export/import/diff/list and related support paths
- `rust/src/sync/`: staged summary, plan, review, audit, preflight, and apply workflows
- `rust/src/common.rs` and `rust/src/http.rs`: shared error, auth, JSON, and transport foundations

Since the original review pass, several recommendations have already started landing:

- release/build policy is more explicit in CI and maintainer build paths
- unified CLI help/example ownership is less centralized than it was
- `sync` has clearer staged-vs-live ownership boundaries
- `sync` blocked-state explanation is stronger
- `sync promotion-preflight` now exists as a first staged environment-promotion contract

The project's strongest product direction is already visible:

- migration and replay
- dependency inspection and governance
- reviewable sync planning instead of blind mutation
- safety-first operator flows

## Strengths

### Clear product direction

The repo has a stronger point of view than a generic Grafana utility.

It is clearly moving toward:

- migration safety
- inspection depth
- governance visibility
- reviewable staged workflows

This is a meaningful differentiator compared with simple export/import tools.

### Broad functional coverage

The maintained Rust surface already covers a wide set of real operator tasks:

- resource inventory
- export/import/diff
- dashboard inspection
- staged sync planning and review
- datasource mutation
- access lifecycle management

This gives the project real platform value even before the next round of improvements.

### Good testing posture

The Rust test surface is broad and active.

The repo also enforces a meaningful quality gate through:

- tests
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`

That is a solid foundation for ongoing refactor work.

### Active modularization effort

The codebase is already being pushed away from monolithic implementation files into smaller helper modules.

That direction is correct and should continue.

## Main Design Risks

### 1. `dashboard` is still the biggest complexity center, and `sync` remains a secondary one

This is the largest structural risk.

Even after the recent module splits, these two areas still carry the most feature density, contract logic, rendering paths, and operator workflow complexity.

The risk is no longer balanced evenly between them.

`sync` has improved materially through staged/live boundary cleanup, explainability work, and the first promotion-preflight workflow. `dashboard` is now the clearer primary hotspot.

The current issue is not just file size. The deeper issue is that several responsibilities are still coupled:

- command orchestration
- contract building
- render decisions
- live-vs-local workflow branching
- review/preflight logic

If this continues, future feature work will keep landing in the same areas and make those domains progressively harder to reason about.

### 2. Public crate boundaries are broader than necessary

`rust/src/lib.rs` exports domain runtime modules, contract modules, helpers, and compatibility re-exports from one crate surface.

That makes the crate play multiple roles at once:

- executable runtime library
- internal shared implementation
- compatibility layer

This is workable for now, but it weakens API discipline and makes it harder to tell which boundaries are truly public and which are just convenient to expose.

### 3. Operator workflows are still stronger as primitives than as end-to-end products

The project already has many strong low-level and mid-level capabilities.

What is still uneven is the completeness of the highest-value operator workflows, especially:

- full environment promotion review/apply workflow
- secret-aware datasource automation
- dashboard-side operator flows that still cross many subsystems

In other words, the building blocks are strong, but some end-to-end stories still feel assembled rather than fully designed.

### 4. Secret handling is now the clearest functional adoption gap

After the recent `sync` and promotion work, secret handling stands out more sharply than it did in the original review.

The project now has stronger staged review, promotion-preflight, and sync explainability, but datasource automation still lacks a first-class explicit contract for secret references and missing-secret handling.

That makes one of the most practical production blockers easier to see as its own roadmap item.

## Design Smells To Watch

These are not all immediate defects, but they are worth tracking:

- too many public or re-exported modules for internal-only behavior
- continued growth of facade modules that still know too much about downstream shape
- dashboard-side workflow logic spreading across inspect/import/governance paths
- promotion work continuing as additive slices without a final review/apply handoff model
- secret-handling semantics remaining conservative but still under-specified as a staged contract

## Three-Phase Improvement Plan

### Phase 1: Consolidate The Wins Already Landed

Goal:

- keep the recently improved `sync` and promotion structure from drifting again

Recommended work:

- document the new promotion mapping and staged contract boundaries more clearly
- keep public-vs-internal crate boundaries under review as helper modules continue to grow
- avoid re-concentrating `sync` logic into orchestration facades now that staged/live ownership is clearer
- treat the recent CI/help/release cleanup as a baseline to preserve, not as solved forever

Definition of success:

- recently improved areas stay stable under follow-on feature work
- new staged promotion work keeps the same contract discipline as sync
- maintainers can explain the current promotion and sync ownership boundaries clearly

### Phase 2: Refactor By Stable Subsystem Boundaries

Goal:

- move from smaller files to clearer ownership boundaries

Recommended work:

- separate `dashboard` into clearer subsystem ownership such as inspect pipeline, governance evaluation, interactive workbench, and import/export support
- keep `sync` promotion work attached to clear staged contract ownership rather than spreading across unrelated modules
- avoid adding new features directly into orchestration facades unless the ownership boundary is already clear

Definition of success:

- `dashboard` no longer dominates the structural risk profile as clearly as it does today
- major domains have identifiable subsystem boundaries instead of only split helper files
- maintainers can predict where new work belongs without re-reading many modules
- contract and workflow ownership are easier to explain than they are today

### Phase 3: Complete The Operator Stories

Goal:

- turn the strongest primitives into stronger end-to-end workflows

Recommended work:

- extend promotion from preflight into a fuller review/apply handoff
- strengthen preflight coverage and rewrite/remap visibility where promotion still depends on manual interpretation
- formalize datasource secret references and explicit secret-missing/secret-loss handling
- keep sync/operator review surfaces coherent instead of fragmenting across too many staged document types

Definition of success:

- users can move from inventory and export to promotion and review with fewer custom steps
- sync and promotion workflows are trusted because failure states are explicit and understandable
- datasource automation becomes more realistic for production use

## Recommended Feature Investment Order

### 1. Inspection and dependency governance

This remains the strongest differentiator.

Priority additions:

- dependency graph export
- blast-radius views
- orphan and stale-resource reporting
- management-friendly HTML or static reports
- stronger governance and quality signals

### 2. Datasource secret handling

This is now the clearest functional gap after the recent sync/promotion progress.

Priority additions:

- placeholder-based secret references
- staged secret contracts
- explicit secret-missing and secret-loss reporting
- optional provider-aware integration that stays reviewable

### 3. Environment promotion

Promotion is still one of the highest-value operator workflows, but it is no longer only a future idea. The next work should build on the staged promotion-preflight and mapping contract already in place.

Priority additions:

- promotion review artifact and handoff model
- resolved remap inventory and warning/blocking separation
- stronger rewrite/preflight coverage
- clearer path from promotion review into eventual controlled apply

### 4. Sync trustworthiness

`sync` has improved materially. The next sync work should be selective, not broad.

Priority additions:

- keep fail-closed behavior strong as promotion and other staged contracts grow
- avoid explainability regressions as new review documents are added
- preserve clear staged/live ownership boundaries
- add more sync depth only when it reinforces operator trust

### 5. Advanced assisted analysis

This should remain exploratory until the core workflow layers are stronger.

Priority additions:

- optional assisted query review
- explainable lint/recommendation output
- local packaging only if it reuses the Rust core cleanly

## Bottom Line

The project is already strong in breadth and direction.

The main challenge is no longer "add more features." The main challenge is:

- making implicit rules explicit
- keeping complexity from re-concentrating in `dashboard` and `sync`
- turning strong primitives into stronger operator workflows

The best next investments are now:

- stronger inspection and dependency visibility
- datasource secret handling as a first-class staged contract
- completing promotion from preflight into fuller review/apply workflow
- keeping sync trustworthy without letting its complexity re-concentrate
- continued architecture simplification around stable subsystem boundaries, especially in `dashboard`
