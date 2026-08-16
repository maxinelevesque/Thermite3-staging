#!/usr/bin/env bash
# Build the Lean spine and require the listed theorem axioms to stay within
# {propext, Classical.choice, Quot.sound}.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LEAN_DIR="$ROOT/lean"

if [ -t 1 ]; then G=$'\033[32m'; R=$'\033[31m'; Z=$'\033[0m'; else G=; R=; Z=; fi
pass() { printf '  %sPASS%s %s\n' "$G" "$Z" "$1"; }
fail() { printf '  %sFAIL%s %s\n' "$R" "$Z" "$1"; }

# Build targets include the spine, its negative pins, and the Stage 3
# reconstruction probe.
IMPORTS=(
  "Thermite.Faithfulness"
  "Thermite.Soundness"
  "Thermite.Exec"
  "Thermite.Exec.Stmt"
  "Thermite.Exec.Loop"
  "Thermite.Relax"
  "Thermite.Strat.Denote"
  "Thermite.PinFiniteEscape"
  "Thermite.Strat.SubstKit"
  "Thermite.PinBrokenLift"
  "Thermite.Strat.Fragment"
  "Thermite.Strat.Cls.Wire"
  "Thermite.Strat.Soundness"
  "Thermite.PinStratFlip"
  "Thermite.PinStratCapture"
  "Thermite.Strat.CombDeriv"
  "Thermite.PinCombDeriv"
  "Thermite.Strat.Restratify"
  "Thermite.PinRestratDropSide"
  "Thermite.Strat.Faithfulness"
  "Thermite.PinStratSelfLoop"
  "Thermite.PinNNFPolarity"
  "Thermite.PinRelaxRefute"
  "Thermite.BvModel"
  "Thermite.PinReconstruction"
  "Thermite.Strat.EprReplay"
  "Thermite.Strat.StructuralInstantiation"
  "Thermite.PinSubstitutionCapture"
  "Thermite.PinSkolemDependencies"
  "Thermite.PinGroundingCompleteness"
  "Thermite.PinInstantiationOmission"
  "Thermite.PinStructuralSkolemScopes"
  "Thermite.PinEprReplay"
  "Thermite.EffectRows"
  "Thermite.LanguageCompleteness"
  "Thermite.PinLanguageNarrowing"
  "Thermite.CheckedTraversal"
  "Thermite.CertificationMetatheory"
  "Thermite.PinCertificationRefinement"
  "Thermite.CertificationOrder"
  "Thermite.PinCertificationOrder"
  "Thermite.CertificationPolicy"
  "Thermite.PinCertificationPolicy"
  "Thermite.CertificationShape"
  "Thermite.ImplementationModel"
  "Thermite.PinImplementationModel"
  "Thermite.TcbDischarge"
  "Thermite.PinTcbDischarge"
)

# Universal soundness theorems and the permanent reconstruction probe.
THEOREMS=(
  "Thermite.lowering_faithful"
  "Thermite.ref_sound"
  "Thermite.Exec.exec_ref_sound"
  "Thermite.Exec.body_ref_sound"
  "Thermite.Exec.while_rule"
  "Thermite.Relax.r_relax_sound"
  "Thermite.Relax.rencode_sound"
  "Thermite.Strat.Cls.classifier_correct"
  "Thermite.Strat.strat_ref_sound"
  "Thermite.Strat.Cls.restrat_conservative"
  "Thermite.Strat.strat_lowering_faithful"
  "Thermite.bv_reconstruction_lrat_probe"
  "Thermite.Strat.Cls.source_false_of_verifiedInstantiation"
  "Thermite.Strat.Cls.source_false_of_verifiedStructuralInstantiation"
  "Thermite.Strat.Cls.source_false_of_verifyEprReplay"
  "Thermite.Strat.Cls.checked_implication_of_problem_unsat"
  "Thermite.Strat.Cls.checked_structural_implication_of_problem_unsat"
  "Thermite.checked_source_is_false"
  "Thermite.EffectRows.call_footprint_closed"
  "Thermite.EffectRows.upper_bound_transitive"
  "Thermite.EffectRows.overlaps_reflexive"
  "Thermite.EffectRows.overlaps_symmetric"
  "Thermite.EffectRows.accepted_pair_commutes"
  "Thermite.EffectRows.disjoint_pairs_impose_no_commutation"
  "Thermite.EffectRows.relational_frame"
  "Thermite.EffectRows.pure_deterministic"
  "Thermite.EffectRows.outside_write_equal"
  "Thermite.LanguageCompleteness.expands_refl"
  "Thermite.LanguageCompleteness.expands_trans"
  "Thermite.LanguageCompleteness.composeGuaranteeComplete"
  "Thermite.LanguageCompleteness.solverRoute_classifies"
  "Thermite.LanguageCompleteness.solverProgress_preserves_membership"
  "Thermite.LanguageCompleteness.finitePolicy_total"
  "Thermite.LanguageCompleteness.policyOutcome_preserves_membership"
  "Thermite.LanguageCompleteness.exact_floor_is_accepted"
  "Thermite.LanguageCompleteness.below_floor_is_rejected"
  "Thermite.LanguageCompleteness.zero_scored_is_outside"
  "Thermite.LanguageCompleteness.over_cap_is_outside"
  "Thermite.LanguageCompleteness.coreV1_expands_to_coreV2"
  "Thermite.LanguageCompleteness.narrowing_is_not_expansion"
  "Thermite.LanguageCompleteness.mutant_silent_narrowing_rejected"
  "Thermite.CheckedTraversal.supportedRFC10_refines_language_fragment"
  "Thermite.CheckedTraversal.verify_sound"
  "Thermite.CheckedTraversal.verify_complete"
  "Thermite.CheckedTraversal.verify_iff_supported"
  "Thermite.CheckedTraversal.produce_complete"
  "Thermite.CheckedTraversal.produce_supported"
  "Thermite.CheckedTraversal.structural_complete_of_verify"
  "Thermite.CheckedTraversal.evidence_well_formed_of_verify"
  "Thermite.CheckedTraversal.footprint_closure_sound_of_verify"
  "Thermite.CheckedTraversal.holding_coverage_sound_of_verify"
  "Thermite.CheckedTraversal.resource_limit_not_certifying"
  "Thermite.CertificationMetatheory.refines_refl"
  "Thermite.CertificationMetatheory.refines_trans"
  "Thermite.CertificationMetatheory.expansion_refines_membership"
  "Thermite.CertificationMetatheory.composition_refines_guarantee"
  "Thermite.CertificationMetatheory.rfc10_producer_refines_certification"
  "Thermite.CertificationMetatheory.frame_refines_refl"
  "Thermite.CertificationMetatheory.frame_refines_trans"
  "Thermite.CertificationMetatheory.bounded_scope_monotone"
  "Thermite.CertificationMetatheory.bounded_two_refines_five"
  "Thermite.CertificationMetatheory.end_to_end_refines_platform"
  "Thermite.CertificationMetatheory.unequal_bound_reverse_rejected"
  "Thermite.CertificationMetatheory.boundary_upgrade_rejected"
  "Thermite.CertificationMetatheory.representative_le_refl"
  "Thermite.CertificationMetatheory.representative_le_antisymm"
  "Thermite.CertificationMetatheory.representative_le_trans"
  "Thermite.CertificationMetatheory.decision_implies_representative_refinement"
  "Thermite.CertificationMetatheory.refinement_implies_decision"
  "Thermite.CertificationMetatheory.representative_decision_iff_refines"
  "Thermite.CertificationMetatheory.solver_accepted_supplies_checked_derivation"
  "Thermite.CertificationMetatheory.solver_accepted_proves_semantic_validity"
  "Thermite.CertificationMetatheory.never_solved_shape_is_not_solver_certificate"
  "Thermite.CertificationMetatheory.representative_population_is_complete"
  "Thermite.CertificationMetatheory.incomparable_branches_have_no_join"
  "Thermite.CertificationMetatheory.incomparable_branches_have_bounded_meet"
  "Thermite.CertificationMetatheory.operation_matrix_covers_all_pairs"
  "Thermite.CertificationMetatheory.incomparable_branches_remain_incomparable"
  "Thermite.CertificationMetatheory.invented_join_rejected"
  "Thermite.CertificationMetatheory.floor_allows_sound"
  "Thermite.CertificationMetatheory.self_validation_sound"
  "Thermite.CertificationMetatheory.policy_population_is_versioned_and_complete"
  "Thermite.CertificationMetatheory.unsound_policy_collapse_rejected"
  "Thermite.CertificationMetatheory.solver_lean_have_no_realizable_join"
  "Thermite.CertificationMetatheory.selected_domain_is_not_five_point"
  "Thermite.CertificationMetatheory.replacement_shape_witness"
  "Thermite.CertificationMetatheory.model_refinement_refl"
  "Thermite.CertificationMetatheory.model_refinement_trans"
  "Thermite.CertificationMetatheory.rustc195_corresponds_on_thermite_fragment"
  "Thermite.CertificationMetatheory.thermite_rust_v1_expands_to_v2"
  "Thermite.CertificationMetatheory.thermite_rust_narrowing_is_explicit"
  "Thermite.CertificationMetatheory.silent_rustc_version_substitution_rejected"
  "Thermite.CertificationMetatheory.silent_rust_fragment_narrowing_rejected"
  "Thermite.CertificationMetatheory.discharge_rust_witness_admitted"
  "Thermite.CertificationMetatheory.rustc195_artifact_corresponds"
  "Thermite.CertificationMetatheory.model_only_retains_rustc"
  "Thermite.CertificationMetatheory.universal_reduction_discharges_exact_rustc"
  "Thermite.CertificationMetatheory.universal_reduction_context_refines"
  "Thermite.CertificationMetatheory.checked_reduction_discharges_exact_rustc"
  "Thermite.CertificationMetatheory.checked_reduction_context_refines"
  "Thermite.CertificationMetatheory.tcbReduction_context_refines"
  "Thermite.CertificationMetatheory.accepted_artifact_sound_despite_incomplete_coverage"
  "Thermite.CertificationMetatheory.illicit_assumption_deletion_rejected"
  "Thermite.CertificationMetatheory.illicit_tcb_reduction_rejected"
  "Thermite.CertificationMetatheory.checked_evidence_is_artifact_pinned"
  "Thermite.CertificationMetatheory.coverage_does_not_supply_correspondence"
  "Thermite.CertificationMetatheory.complete_coverage_cannot_mask_version_substitution"
)
ALLOWED="propext Classical.choice Quot.sound"

if ! command -v lake >/dev/null 2>&1; then
  echo "lean-axiom-probe: lake not found on PATH" >&2
  exit 2
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "  building the Lean spine (lake build) ..."
if ! (cd "$LEAN_DIR" && lake build "${IMPORTS[@]}") >"$TMP/build.log" 2>&1; then
  fail "lake build failed"
  tail -15 "$TMP/build.log" | sed 's/^/      /'
  exit 2
fi
pass "lake build succeeded"

PROBE="$TMP/axprobe.lean"
{
  for m in "${IMPORTS[@]}"; do echo "import $m"; done
  for t in "${THEOREMS[@]}"; do echo "#print axioms $t"; done
} >"$PROBE"

AX_OUT="$( (cd "$LEAN_DIR" && lake env lean "$PROBE") 2>&1 )"
AX_RC=$?
if [ "$AX_RC" -ne 0 ]; then
  fail "the axiom probe failed to elaborate (lake env lean exited $AX_RC)"
  echo "$AX_OUT" | tail -10 | sed 's/^/      /'
  exit 2
fi

THM_FAIL=0
for t in "${THEOREMS[@]}"; do
  # Lean may wrap the report, so capture through its closing bracket.
  report="$(
    printf '%s\n' "$AX_OUT" | awk -v target="'$t'" '
      index($0, target) { found = 1 }
      found {
        print
        if (index($0, "does not depend on any axioms") || index($0, "]")) exit
      }
    '
  )"
  if [ -z "$report" ]; then
    fail "$t — no axiom report found"
    THM_FAIL=1
    continue
  fi
  axlist="$(
    printf '%s\n' "$report" |
      tr '\n' ' ' |
      sed -n 's/.*\[\(.*\)\].*/\1/p' |
      tr ',' '\n' |
      sed 's/[[:space:]]//g'
  )"
  bad=""
  while IFS= read -r ax; do
    [ -z "$ax" ] && continue
    case " $ALLOWED " in
      *" $ax "*) : ;;
      *) bad="$bad $ax" ;;
    esac
  done <<<"$axlist"
  if [ -n "$bad" ]; then
    fail "$t — disallowed axiom(s):$bad"
    THM_FAIL=1
  else
    pass "$t — axioms ⊆ {propext, Classical.choice, Quot.sound}"
  fi
done

[ "$THM_FAIL" -eq 0 ] || exit 1
exit 0
