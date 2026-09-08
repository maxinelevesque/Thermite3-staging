# Post-Thermite-3 execution practice

This note is the operational companion to `.design/post-t3-roadmap.md`. It
records the workflow rules that apply to the process-cleanup slice, issues
#55–#57, alpha.11, and the subsequent roadmap reassessment.

## Tiered qualification

During implementation, run the smallest focused tests and structural gates that
cover the current slice. Before review, stabilize the tree, materialize affected
content-bound claim receipts once, refresh generated status and audit pins, and
run full qualification. Commit the exact qualified tree before requesting an
external review. A changed reviewed head invalidates the review receipt and must
repeat the affected qualification and exact-head review.

Claim materialization performs each positive and counterfeit observation in the
claim author, then validates the rendered registry and ledger structurally before
atomic publication. It must not immediately replay the same expensive verifier
population serially inside the renderer; independent replay belongs to final
qualification and CI.

Private repository or diff content may be sent to an external reviewer only
after fresh explicit user approval for that scope and head. Approval for one RFC,
issue, reviewer, or commit does not carry into another.

## Workspace-version changes

The workspace version is part of the content bound by claim-closure receipts.
A change to `Cargo.toml` or `Cargo.lock` therefore requires a planned receipt
refresh even when the release is otherwise version-only. The authoritative
release diff may include the generated registry, completeness ledger, language
inventory pin, and audited design pins produced by that refresh. Treating those
updates as an unexpected CI repair is a process failure.

Alpha releases follow the established repository practice unless explicitly
changed: merge a dedicated release PR to `main`, verify locked metadata and the
full required CI surface, and do not create a tag or GitHub Release.

## Low-churn monitoring

Prefer event-driven completion waits when the environment exposes them. When
polling is unavoidable:

1. start with a short interval while a newly launched action is expected to
   change quickly;
2. double the interval after unchanged observations, capped at 30 minutes;
3. reset the interval only after a state transition;
4. stop polling when user input arrives or the action reaches a terminal state;
5. do not emit user-facing commentary for unchanged state.

Notify only for a meaningful milestone, a failed check requiring diagnosis, a
material direction choice, a required external-sharing approval, or completion.
Queue delay and an unchanged pending state are not milestones.

## Merge boundary

Before merge, verify that the PR head equals the qualified and reviewed head,
every required check is green or intentionally skipped by reviewed path logic,
and the target branch is the roadmap's declared branch. After merge, verify the
merge commit on `main`, the intended issue/milestone transition, and a clean
local checkout synchronized with `origin/main`.
