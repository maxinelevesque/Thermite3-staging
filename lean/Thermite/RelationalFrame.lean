/-
  Thermite/RelationalFrame.lean — the semantic Tier-A relational frame kernel.

  Unlike the historical wrapper in `Thermite.EffectRows`, this module derives
  result congruence and write framing by induction over an executable region
  program.  The language is the canonical state-effect core used to establish
  the universal theorem before per-artifact replay and source-to-target
  transport are attached.
-/

import Thermite.EffectRows
import Thermite.Exec.Stmt

namespace Thermite.RelationalFrame

open Thermite.EffectRows

abbrev Region := EffectRows.Region
abbrev Store := Region → Nat

def Store.write (store : Store) (region : Region) (value : Nat) : Store :=
  fun target => if target = region then value else store target

theorem Store.write_at (store : Store) (region : Region) (value : Nat) :
    Store.write store region value region = value := by
  simp [Store.write]

theorem Store.write_away (store : Store) (region target : Region) (value : Nat)
    (different : target ≠ region) :
    Store.write store region value target = store target := by
  simp [Store.write, different]

def AgreesOn (left right : Store) (regions : List Region) : Prop :=
  ∀ region ∈ regions, left region = right region

inductive Expr where
  | literal (value : Nat)
  | region (name : Region)
  | add (left right : Expr)
  | multiply (left right : Expr)
deriving DecidableEq, Repr

namespace Expr

def reads : Expr → List Region
  | .literal _ => []
  | .region name => [name]
  | .add left right => left.reads ++ right.reads
  | .multiply left right => left.reads ++ right.reads

def eval : Expr → Store → Nat
  | .literal value, _ => value
  | .region name, store => store name
  | .add left right, store => left.eval store + right.eval store
  | .multiply left right, store => left.eval store * right.eval store

theorem eval_congruent : ∀ (expr : Expr) {left right : Store},
    AgreesOn left right expr.reads → expr.eval left = expr.eval right
  | .literal _, _, _, _ => rfl
  | .region name, _, _, agrees => agrees name (by simp [reads])
  | .add left right, _, _, agrees => by
      simp only [eval]
      rw [eval_congruent left (fun region member =>
            agrees region (by simp [reads, member])),
          eval_congruent right (fun region member =>
            agrees region (by simp [reads, member]))]
  | .multiply left right, _, _, agrees => by
      simp only [eval]
      rw [eval_congruent left (fun region member =>
            agrees region (by simp [reads, member])),
          eval_congruent right (fun region member =>
            agrees region (by simp [reads, member]))]

end Expr

inductive Outcome where
  | returned (value : Nat)
  | exceptional
deriving DecidableEq, Repr

inductive Program where
  | ret (value : Expr)
  | write (region : Region) (value : Expr) (next : Program)
  | branchZero (condition : Expr) (thenProgram elseProgram : Program)
  | raise
  | diverge
deriving DecidableEq, Repr

namespace Program

def reads : Program → List Region
  | .ret value => value.reads
  | .write _ value next => value.reads ++ next.reads
  | .branchZero condition thenProgram elseProgram =>
      condition.reads ++ thenProgram.reads ++ elseProgram.reads
  | .raise => []
  | .diverge => []

def writes : Program → List Region
  | .ret _ => []
  | .write region _ next => region :: next.writes
  | .branchZero _ thenProgram elseProgram => thenProgram.writes ++ elseProgram.writes
  | .raise => []
  | .diverge => []

def footprint (program : Program) : Footprint :=
  (program.reads.map fun region =>
    ({ operation := .read, region := region } : EffectRows.Effect)) ++
  (program.writes.map fun region =>
    ({ operation := .write, region := region } : EffectRows.Effect))

def writesRegion (program : Program) (target : Region) : Prop :=
  ∃ region ∈ program.writes, Overlaps region target

def run : Program → Store → Option (Outcome × Store)
  | .ret value, store => some (.returned (value.eval store), store)
  | .write region value next, store =>
      next.run (store.write region (value.eval store))
  | .branchZero condition thenProgram elseProgram, store =>
      if condition.eval store = 0 then thenProgram.run store else elseProgram.run store
  | .raise, store => some (.exceptional, store)
  | .diverge, _ => none

theorem footprint_agreement_implies_reads {program : Program} {left right : Store}
    (agrees : AgreesOnFootprint left right program.footprint) :
    AgreesOn left right program.reads := by
  intro region member
  exact agrees ⟨.read, region⟩ (by simp [footprint, member])

theorem run_frames_writes : ∀ (program : Program) {start : Store}
    {outcome : Outcome} {finish : Store},
    program.run start = some (outcome, finish) →
      ∀ target, ¬ program.writesRegion target → finish target = start target
  | .ret _, _, _, _, execution, target, _ => by
      simp only [run] at execution
      cases execution
      rfl
  | .write region value next, start, outcome, finish, execution, target, outside => by
      have outsideNext : ¬ next.writesRegion target := by
        intro affected
        apply outside
        rcases affected with ⟨written, member, overlap⟩
        exact ⟨written, by simp [writes, member], overlap⟩
      have different : target ≠ region := by
        intro equal
        apply outside
        subst target
        exact ⟨region, by simp [writes], overlaps_reflexive region⟩
      rw [run_frames_writes next execution target outsideNext,
          Store.write_away start region target (value.eval start) different]
  | .branchZero condition thenProgram elseProgram,
      start, outcome, finish, execution, target, outside => by
      by_cases zero : condition.eval start = 0
      · have thenExecution : thenProgram.run start = some (outcome, finish) := by
          simpa [run, zero] using execution
        exact run_frames_writes thenProgram thenExecution target (by
          intro affected
          apply outside
          rcases affected with ⟨written, member, overlap⟩
          exact ⟨written, by simp [writes, member], overlap⟩)
      · have elseExecution : elseProgram.run start = some (outcome, finish) := by
          simpa [run, zero] using execution
        exact run_frames_writes elseProgram elseExecution target (by
          intro affected
          apply outside
          rcases affected with ⟨written, member, overlap⟩
          exact ⟨written, by simp [writes, member], overlap⟩)
  | .raise, _, _, _, execution, target, _ => by
      simp only [run] at execution
      cases execution
      rfl
  | .diverge, _, _, _, execution, _, _ => by
      simp [run] at execution

theorem run_result_congruent : ∀ (program : Program) {left right : Store}
    {leftOutcome rightOutcome : Outcome} {leftFinish rightFinish : Store},
    AgreesOn left right program.reads →
    program.run left = some (leftOutcome, leftFinish) →
    program.run right = some (rightOutcome, rightFinish) →
    leftOutcome = rightOutcome
  | .ret value, _, _, _, _, _, _, agrees, leftRun, rightRun => by
      simp only [run] at leftRun rightRun
      cases leftRun
      cases rightRun
      simp [Expr.eval_congruent value agrees]
  | .write region value next, left, right, _, _, _, _, agrees, leftRun, rightRun => by
      have valueEqual : value.eval left = value.eval right :=
        Expr.eval_congruent value (fun target member =>
          agrees target (by simp [reads, member]))
      have nextAgrees : AgreesOn
          (left.write region (value.eval left))
          (right.write region (value.eval right)) next.reads := by
        intro target member
        by_cases same : target = region
        · subst target
          simp [Store.write, valueEqual]
        · simp [Store.write, same]
          exact agrees target (by simp [reads, member])
      exact run_result_congruent next nextAgrees leftRun rightRun
  | .branchZero condition thenProgram elseProgram,
      left, right, _, _, _, _, agrees, leftRun, rightRun => by
      have conditionEqual : condition.eval left = condition.eval right :=
        Expr.eval_congruent condition (fun target member =>
          agrees target (by simp [reads, member]))
      simp only [run] at leftRun rightRun
      rw [← conditionEqual] at rightRun
      by_cases zero : condition.eval left = 0
      · exact run_result_congruent thenProgram
          (fun target member => agrees target (by simp [reads, member]))
          (by simpa [zero] using leftRun)
          (by simpa [zero] using rightRun)
      · exact run_result_congruent elseProgram
          (fun target member => agrees target (by simp [reads, member]))
          (by simpa [zero] using leftRun)
          (by simpa [zero] using rightRun)
  | .raise, _, _, _, _, _, _, _, leftRun, rightRun => by
      simp only [run] at leftRun rightRun
      cases leftRun
      cases rightRun
      rfl
  | .diverge, _, _, _, _, _, _, _, leftRun, _ => by
      simp [run] at leftRun

/-- Tier A's derived two-run theorem. Neither result congruence nor write
    framing is a premise: both follow from the program's executable semantics. -/
theorem relational_frame
    (program : Program)
    {left right : Store}
    {leftOutcome rightOutcome : Outcome}
    {leftFinish rightFinish : Store}
    (agrees : AgreesOnFootprint left right program.footprint)
    (leftRun : program.run left = some (leftOutcome, leftFinish))
    (rightRun : program.run right = some (rightOutcome, rightFinish)) :
    leftOutcome = rightOutcome ∧
      (∀ target, ¬ program.writesRegion target →
        leftFinish target = left target ∧ rightFinish target = right target) := by
  constructor
  · exact run_result_congruent program
      (footprint_agreement_implies_reads agrees) leftRun rightRun
  · intro target outside
    exact ⟨run_frames_writes program leftRun target outside,
      run_frames_writes program rightRun target outside⟩

theorem outside_write_equal
    (program : Program)
    {left right : Store}
    {leftOutcome rightOutcome : Outcome}
    {leftFinish rightFinish : Store}
    {target : Region}
    (initiallyEqual : left target = right target)
    (outside : ¬ program.writesRegion target)
    (leftRun : program.run left = some (leftOutcome, leftFinish))
    (rightRun : program.run right = some (rightOutcome, rightFinish)) :
    leftFinish target = rightFinish target := by
  rw [run_frames_writes program leftRun target outside,
      run_frames_writes program rightRun target outside]
  exact initiallyEqual

end Program

inductive ResearchGate where
  | probabilisticDenotation
  | peerProgress
  | externalCoupling
  | terminationWitness
deriving DecidableEq, Repr

inductive Support where
  | derived
  | conditional (gate : ResearchGate)
  | unavailable (gate : ResearchGate)
  | structural
  | notApplicable
deriving DecidableEq, Repr

structure ProjectionSupport where
  result : Support
  writeFrame : Support
  outcome : Support
  termination : Support
  trace : Support
  accumulator : Support
deriving DecidableEq, Repr

/-- The closed effect-theory basis, including the two deliberately unmodelled
    atoms.  This is separate from `EffectKind`: surface `rand`, for example,
    is the deterministic state instance `state(entropy)`, not bare `random`. -/
inductive PrimitiveEffect where
  | state
  | accrues
  | exception
  | partiality
  | io
  | random
  | blocks
deriving DecidableEq, Repr

def stateSupport : ProjectionSupport :=
  ⟨.derived, .derived, .derived, .derived, .notApplicable, .notApplicable⟩

def supportForPrimitive : PrimitiveEffect → ProjectionSupport
  | .state => stateSupport
  | .accrues =>
      ⟨.derived, .derived, .derived, .derived, .notApplicable, .derived⟩
  | .exception =>
      ⟨.notApplicable, .derived, .derived, .notApplicable,
        .notApplicable, .notApplicable⟩
  | .partiality =>
      ⟨.derived, .derived, .derived, .unavailable .terminationWitness,
        .notApplicable, .notApplicable⟩
  | .io =>
      ⟨.conditional .externalCoupling, .derived, .conditional .externalCoupling,
        .derived, .conditional .externalCoupling, .notApplicable⟩
  | .random =>
      ⟨.unavailable .probabilisticDenotation, .unavailable .probabilisticDenotation,
        .unavailable .probabilisticDenotation, .unavailable .probabilisticDenotation,
        .unavailable .probabilisticDenotation, .notApplicable⟩
  | .blocks =>
      ⟨.unavailable .peerProgress, .unavailable .peerProgress,
        .unavailable .peerProgress, .unavailable .peerProgress,
        .unavailable .peerProgress, .notApplicable⟩

def allPrimitiveEffects : List PrimitiveEffect :=
  [.state, .accrues, .exception, .partiality, .io, .random, .blocks]

theorem allPrimitiveEffects_complete (effect : PrimitiveEffect) :
    effect ∈ allPrimitiveEffects := by
  cases effect <;> simp [allPrimitiveEffects]

theorem allPrimitiveEffects_nodup : allPrimitiveEffects.Nodup := by
  decide

/-- Independent projection composition.  A missing theorem suppresses only
    that projection; it does not collapse the other five coordinates. -/
def combineSupport : Support → Support → Support
  | .unavailable gate, _ => .unavailable gate
  | _, .unavailable gate => .unavailable gate
  | .conditional gate, _ => .conditional gate
  | _, .conditional gate => .conditional gate
  | .notApplicable, support => support
  | support, .notApplicable => support
  | .structural, .structural => .structural
  | .structural, .derived => .derived
  | .derived, .structural => .derived
  | .derived, .derived => .derived

def combineProjection (left right : ProjectionSupport) : ProjectionSupport :=
  ⟨combineSupport left.result right.result,
    combineSupport left.writeFrame right.writeFrame,
    combineSupport left.outcome right.outcome,
    combineSupport left.termination right.termination,
    combineSupport left.trace right.trace,
    combineSupport left.accumulator right.accumulator⟩

inductive EffectKind where
  | read
  | write
  | net
  | alloc
  | time
  | rand
  | panic
  | diverge
  | term
  | owns
  | forgets
  | blocks
deriving DecidableEq, Repr

def supportForKind : EffectKind → ProjectionSupport
  | .read => stateSupport
  | .write => stateSupport
  | .net => combineProjection stateSupport (supportForPrimitive .io)
  | .alloc => stateSupport
  | .time => stateSupport
  | .rand => stateSupport
  | .panic =>
      ⟨.notApplicable, .derived, .derived, .notApplicable, .notApplicable, .notApplicable⟩
  | .diverge =>
      ⟨.conditional .terminationWitness, .conditional .terminationWitness,
        .conditional .terminationWitness, .unavailable .terminationWitness,
        .notApplicable, .notApplicable⟩
  | .term => stateSupport
  | .owns =>
      ⟨.structural, .structural, .structural, .structural, .structural, .structural⟩
  | .forgets =>
      ⟨.structural, .structural, .structural, .structural, .structural, .structural⟩
  | .blocks =>
      ⟨.unavailable .peerProgress, .unavailable .peerProgress,
        .unavailable .peerProgress, .unavailable .peerProgress,
        .unavailable .peerProgress, .notApplicable⟩

def allEffectKinds : List EffectKind :=
  [.read, .write, .net, .alloc, .time, .rand, .panic, .diverge,
    .term, .owns, .forgets, .blocks]

theorem allEffectKinds_complete (kind : EffectKind) : kind ∈ allEffectKinds := by
  cases kind <;> simp [allEffectKinds]

theorem allEffectKinds_nodup : allEffectKinds.Nodup := by
  decide

theorem net_retains_write_frame :
    (supportForKind .net).result = .conditional .externalCoupling ∧
      (supportForKind .net).writeFrame = .derived := by
  decide

theorem net_is_state_plus_io :
    supportForKind .net =
      combineProjection (supportForPrimitive .state) (supportForPrimitive .io) := rfl

theorem bare_random_requires_new_denotation :
    (supportForPrimitive .random).result = .unavailable .probabilisticDenotation ∧
      (supportForPrimitive .random).writeFrame =
        .unavailable .probabilisticDenotation := by
  decide

theorem blocks_requires_peer_progress :
    (supportForPrimitive .blocks).result = .unavailable .peerProgress ∧
      (supportForPrimitive .blocks).termination = .unavailable .peerProgress := by
  decide

/- Non-vacuity fixtures.  The two initial stores agree on the program's read
   footprint while disagreeing elsewhere; a third store disagrees on a read.
   The examples pin ordinary return, exception, partial execution, permitted
   writes, forbidden outside writes, and conservative region ancestry. -/
namespace Examples

def regionA : Region := [0]
def regionB : Region := [1]
def childOfA : Region := [0, 1]

def leftStore : Store := fun region =>
  if region = regionA then 4 else if region = regionB then 9 else 0

def rightStore : Store := fun region =>
  if region = regionA then 4 else if region = regionB then 27 else 0

def unequalReadStore : Store := fun region =>
  if region = regionA then 7 else if region = regionB then 9 else 0

def incrementA : Program :=
  .write regionA (.add (.region regionA) (.literal 1))
    (.ret (.region regionA))

theorem successful_left :
    incrementA.run leftStore =
      some (.returned 5, leftStore.write regionA 5) := by
  rfl

theorem successful_right :
    incrementA.run rightStore =
      some (.returned 5, rightStore.write regionA 5) := by
  rfl

theorem equal_initial_footprint :
    AgreesOnFootprint leftStore rightStore incrementA.footprint := by
  intro effect member
  simp [incrementA, Program.footprint, Program.reads, Program.writes,
    Expr.reads] at member
  rcases member with rfl | rfl | rfl <;> decide

theorem successful_pair_has_equal_result :
    (Outcome.returned 5 : Outcome) = .returned 5 ∧
      ∀ target, ¬ incrementA.writesRegion target →
        (leftStore.write regionA 5) target = leftStore target ∧
        (rightStore.write regionA 5) target = rightStore target := by
  exact Program.relational_frame incrementA equal_initial_footprint
    successful_left successful_right

theorem permitted_write : incrementA.writesRegion regionA := by
  exact ⟨regionA, by simp [incrementA, Program.writes], overlaps_reflexive regionA⟩

theorem forbidden_outside_write : ¬ incrementA.writesRegion regionB := by
  simp [incrementA, Program.writes, Program.writesRegion, regionA, regionB,
    Overlaps]

theorem outside_region_is_preserved :
    (leftStore.write regionA 5) regionB = leftStore regionB := by
  exact Store.write_away leftStore regionA regionB 5 (by decide)

theorem unequal_read_footprint_changes_result :
    incrementA.run unequalReadStore =
      some (.returned 8, unequalReadStore.write regionA 8) := by
  rfl

theorem exceptional_outcome :
    Program.raise.run leftStore = some (.exceptional, leftStore) := rfl

theorem partial_execution : Program.diverge.run leftStore = none := rfl

theorem parent_write_covers_child : incrementA.writesRegion childOfA := by
  refine ⟨regionA, by simp [incrementA, Program.writes], ?_⟩
  exact Or.inl (by simp [regionA, childOfA])

end Examples

/-! ## Canonical bounded carrier

`Program` above is the small executable proof fixture.  The certificate-facing
carrier below extends Thermite's actual bounded body state with declared region
state.  Local expressions therefore retain `ExecVal`, overflow obligations,
casts, and partial evaluation from `Thermite.Exec`; the relational theorem adds
only region framing and explicit terminal outcomes. -/

namespace Bounded

structure World where
  locals : Thermite.Exec.State
  regions : Region → Thermite.Exec.ExecVal

def World.write (world : World) (region : Region)
    (value : Thermite.Exec.ExecVal) : World :=
  { world with regions := fun target => if target = region then value else world.regions target }

theorem World.write_at (world : World) (region : Region)
    (value : Thermite.Exec.ExecVal) :
    (world.write region value).regions region = value := by
  simp [World.write]

theorem World.write_away (world : World) (region target : Region)
    (value : Thermite.Exec.ExecVal) (different : target ≠ region) :
    (world.write region value).regions target = world.regions target := by
  simp [World.write, different]

def Agrees (left right : World) (regions : List Region) : Prop :=
  left.locals = right.locals ∧
    ∀ region ∈ regions, left.regions region = right.regions region

inductive Expr where
  | literal (value : Thermite.Exec.ExecVal)
  | local (value : Thermite.Exec.ExecExpr)
  | region (name : Region)
  | arith (op : Thermite.Exec.AOp) (left right : Expr)
deriving DecidableEq, Repr

namespace Expr

def reads : Expr → List Region
  | .literal _ => []
  | .local _ => []
  | .region name => [name]
  | .arith _ left right => left.reads ++ right.reads

def eval : Expr → World → Option Thermite.Exec.ExecVal
  | .literal value, _ => some value
  | .local value, world => Thermite.Exec.execDenote value world.locals.env
  | .region name, world => some (world.regions name)
  | .arith op left right, world => do
      let leftValue ← Thermite.Exec.asInt (← left.eval world)
      let rightValue ← Thermite.Exec.asInt (← right.eval world)
      let result ← Thermite.Exec.evalArith op leftValue rightValue
      some (.int result)

theorem eval_congruent : ∀ (expr : Expr) {left right : World},
    Agrees left right expr.reads → expr.eval left = expr.eval right
  | .literal _, _, _, _ => rfl
  | .local value, left, right, agrees => by
      simp only [eval]
      rw [agrees.1]
  | .region name, _, _, agrees => by
      simp only [eval]
      rw [agrees.2 name (by simp [reads])]
  | .arith op leftExpr rightExpr, left, right, agrees => by
      have leftEqual := eval_congruent leftExpr (left := left) (right := right)
        ⟨agrees.1, fun region member => agrees.2 region (by simp [reads, member])⟩
      have rightEqual := eval_congruent rightExpr (left := left) (right := right)
        ⟨agrees.1, fun region member => agrees.2 region (by simp [reads, member])⟩
      simp only [eval]
      rw [leftEqual, rightEqual]

end Expr

inductive Outcome where
  | returned (value : Thermite.Exec.ExecVal)
  | exceptional
deriving DecidableEq, Repr

inductive Program where
  | ret (value : Expr)
  | write (region : Region) (value : Expr) (next : Program)
  | branch (condition : Expr) (thenProgram elseProgram : Program)
  | raise
  | diverge
deriving DecidableEq, Repr

namespace Program

def reads : Program → List Region
  | .ret value => value.reads
  | .write _ value next => value.reads ++ next.reads
  | .branch condition thenProgram elseProgram =>
      condition.reads ++ thenProgram.reads ++ elseProgram.reads
  | .raise | .diverge => []

def writes : Program → List Region
  | .ret _ => []
  | .write region _ next => region :: next.writes
  | .branch _ thenProgram elseProgram => thenProgram.writes ++ elseProgram.writes
  | .raise | .diverge => []

def footprint (program : Program) : Footprint :=
  (program.reads.map fun region =>
    ({ operation := .read, region := region } : EffectRows.Effect)) ++
  (program.writes.map fun region =>
    ({ operation := .write, region := region } : EffectRows.Effect))

def writesRegion (program : Program) (target : Region) : Prop :=
  ∃ region ∈ program.writes, Overlaps region target

def AgreesOnEffectFootprint (left right : Region → Thermite.Exec.ExecVal)
    (effects : Footprint) : Prop :=
  ∀ effect ∈ effects, left effect.region = right effect.region

def run : Program → World → Option (Outcome × World)
  | .ret value, world => do
      let result ← value.eval world
      some (.returned result, world)
  | .write region value next, world => do
      let result ← value.eval world
      next.run (world.write region result)
  | .branch condition thenProgram elseProgram, world => do
      let value ← condition.eval world
      let selected ← Thermite.Exec.asBool value
      if selected then thenProgram.run world else elseProgram.run world
  | .raise, world => some (.exceptional, world)
  | .diverge, _ => none

theorem footprint_agreement {program : Program} {left right : World}
    (agrees : left.locals = right.locals ∧
      AgreesOnEffectFootprint left.regions right.regions program.footprint) :
    Agrees left right program.reads := by
  refine ⟨agrees.1, ?_⟩
  intro region member
  exact agrees.2 ⟨.read, region⟩ (by simp [footprint, member])

theorem run_frames_writes : ∀ (program : Program) {start : World}
    {outcome : Outcome} {finish : World},
    program.run start = some (outcome, finish) →
      ∀ target, ¬ program.writesRegion target →
        finish.regions target = start.regions target
  | .ret value, start, outcome, finish, execution, target, _ => by
      simp only [run] at execution
      cases evaluated : value.eval start with
      | none => simp [evaluated] at execution
      | some result =>
          rw [evaluated] at execution
          exact (congrArg (fun pair => pair.2.regions target)
            (Option.some.inj execution)).symm
  | .write region value next, start, outcome, finish, execution, target, outside => by
      have outsideNext : ¬ next.writesRegion target := by
        intro affected
        apply outside
        rcases affected with ⟨written, member, overlap⟩
        exact ⟨written, by simp [writes, member], overlap⟩
      have different : target ≠ region := by
        intro equal
        apply outside
        subst target
        exact ⟨region, by simp [writes], overlaps_reflexive region⟩
      simp only [run] at execution
      cases evaluated : value.eval start with
      | none => simp [evaluated] at execution
      | some result =>
          rw [evaluated] at execution
          rw [run_frames_writes next execution target outsideNext,
            World.write_away start region target result different]
  | .branch condition thenProgram elseProgram,
      start, outcome, finish, execution, target, outside => by
      simp only [run] at execution
      cases evaluated : condition.eval start <;> simp [evaluated] at execution
      case some value =>
        cases boolean : Thermite.Exec.asBool value <;> simp [boolean] at execution
        case some selected =>
          cases selected
          · exact run_frames_writes elseProgram execution target (by
              intro affected
              apply outside
              rcases affected with ⟨written, member, overlap⟩
              exact ⟨written, by simp [writes, member], overlap⟩)
          · exact run_frames_writes thenProgram execution target (by
              intro affected
              apply outside
              rcases affected with ⟨written, member, overlap⟩
              exact ⟨written, by simp [writes, member], overlap⟩)
  | .raise, _, _, _, execution, _, _ => by
      simp only [run] at execution
      cases execution
      rfl
  | .diverge, _, _, _, execution, _, _ => by
      simp [run] at execution

theorem run_result_congruent : ∀ (program : Program) {left right : World}
    {leftOutcome rightOutcome : Outcome} {leftFinish rightFinish : World},
    Agrees left right program.reads →
    program.run left = some (leftOutcome, leftFinish) →
    program.run right = some (rightOutcome, rightFinish) →
    leftOutcome = rightOutcome
  | .ret value, left, right, _, _, _, _, agrees, leftRun, rightRun => by
      have valueEqual := Expr.eval_congruent value agrees
      simp only [run] at leftRun rightRun
      rw [valueEqual] at leftRun
      cases evaluated : value.eval right with
      | none => simp [evaluated] at leftRun
      | some result =>
          rw [evaluated] at leftRun rightRun
          have leftOutcomeEq := congrArg Prod.fst (Option.some.inj leftRun)
          have rightOutcomeEq := congrArg Prod.fst (Option.some.inj rightRun)
          exact leftOutcomeEq.symm.trans rightOutcomeEq
  | .write region value next, left, right, _, _, _, _, agrees, leftRun, rightRun => by
      have valueEqual := Expr.eval_congruent value
        ⟨agrees.1, fun target member => agrees.2 target (by simp [reads, member])⟩
      simp only [run] at leftRun rightRun
      rw [valueEqual] at leftRun
      cases evaluated : value.eval right with
      | none => simp [evaluated] at leftRun
      | some result =>
          rw [evaluated] at leftRun rightRun
          have nextAgrees : Agrees (left.write region result)
              (right.write region result) next.reads := by
            constructor
            · exact agrees.1
            · intro target member
              by_cases same : target = region
              · subst target
                simp [World.write]
              · simp [World.write, same]
                exact agrees.2 target (by simp [reads, member])
          exact run_result_congruent next nextAgrees leftRun rightRun
  | .branch condition thenProgram elseProgram,
      left, right, _, _, _, _, agrees, leftRun, rightRun => by
      have conditionEqual := Expr.eval_congruent condition
        ⟨agrees.1, fun target member => agrees.2 target (by simp [reads, member])⟩
      simp only [run] at leftRun rightRun
      rw [conditionEqual] at leftRun
      cases evaluated : condition.eval right with
      | none => simp [evaluated] at leftRun
      | some value =>
          rw [evaluated] at leftRun rightRun
          cases boolean : Thermite.Exec.asBool value with
          | none => simp [boolean] at leftRun
          | some selected =>
              cases selected
              · simp [boolean] at leftRun rightRun
                exact run_result_congruent elseProgram
                  ⟨agrees.1, fun target member =>
                    agrees.2 target (by simp [reads, member])⟩ leftRun rightRun
              · simp [boolean] at leftRun rightRun
                exact run_result_congruent thenProgram
                  ⟨agrees.1, fun target member =>
                    agrees.2 target (by simp [reads, member])⟩ leftRun rightRun
  | .raise, _, _, _, _, _, _, _, leftRun, rightRun => by
      simp only [run] at leftRun rightRun
      cases leftRun
      cases rightRun
      rfl
  | .diverge, _, _, _, _, _, _, _, leftRun, _ => by
      simp [run] at leftRun

theorem relational_frame
    (program : Program)
    {left right : World}
    {leftOutcome rightOutcome : Outcome}
    {leftFinish rightFinish : World}
    (agrees : left.locals = right.locals ∧
      AgreesOnEffectFootprint left.regions right.regions program.footprint)
    (leftRun : program.run left = some (leftOutcome, leftFinish))
    (rightRun : program.run right = some (rightOutcome, rightFinish)) :
    leftOutcome = rightOutcome ∧
      (∀ target, ¬ program.writesRegion target →
        leftFinish.regions target = left.regions target ∧
        rightFinish.regions target = right.regions target) := by
  constructor
  · exact run_result_congruent program (footprint_agreement agrees) leftRun rightRun
  · intro target outside
    exact ⟨run_frames_writes program leftRun target outside,
      run_frames_writes program rightRun target outside⟩

end Program

namespace Examples

def regionA : Region := [0]
def regionB : Region := [1]

def localState : Thermite.Exec.State :=
  { env :=
      { vars := fun _ => .bool false
        slices := fun _ => [] }
    scope := fun _ => false }

def leftRegions : Region → Thermite.Exec.ExecVal := fun region =>
  if region = regionA then .bool false else .bool false

def rightRegions : Region → Thermite.Exec.ExecVal := fun region =>
  if region = regionA then .bool false else .bool true

def unequalRegions : Region → Thermite.Exec.ExecVal := fun region =>
  if region = regionA then .bool true else .bool false

def leftWorld : World := ⟨localState, leftRegions⟩
def rightWorld : World := ⟨localState, rightRegions⟩
def unequalWorld : World := ⟨localState, unequalRegions⟩

def setA : Program :=
  .write regionA (.literal (.bool true)) (.ret (.region regionA))

theorem setA_left_run :
    setA.run leftWorld =
      some (.returned (.bool true), leftWorld.write regionA (.bool true)) := rfl

theorem setA_right_run :
    setA.run rightWorld =
      some (.returned (.bool true), rightWorld.write regionA (.bool true)) := rfl

theorem setA_initial_agreement :
    leftWorld.locals = rightWorld.locals ∧
      Program.AgreesOnEffectFootprint leftWorld.regions rightWorld.regions setA.footprint := by
  constructor
  · rfl
  · intro effect member
    simp [setA, Program.footprint, Program.reads, Program.writes, Expr.reads] at member
    rcases member with rfl | rfl <;> decide

theorem setA_relational_frame :
    (Outcome.returned (.bool true) : Outcome) = .returned (.bool true) ∧
      ∀ target, ¬ setA.writesRegion target →
        (leftWorld.write regionA (.bool true)).regions target = leftWorld.regions target ∧
        (rightWorld.write regionA (.bool true)).regions target = rightWorld.regions target := by
  exact Program.relational_frame setA setA_initial_agreement setA_left_run setA_right_run

theorem unequal_read_changes_result :
    Program.run (.ret (.region regionA)) leftWorld =
      some (.returned (.bool false), leftWorld) ∧
    Program.run (.ret (.region regionA)) unequalWorld =
      some (.returned (.bool true), unequalWorld) := by
  constructor <;> rfl

theorem exceptional_outcome :
    Program.raise.run leftWorld = some (.exceptional, leftWorld) := rfl

theorem partial_execution : Program.diverge.run leftWorld = none := rfl

end Examples

end Bounded

end Thermite.RelationalFrame
