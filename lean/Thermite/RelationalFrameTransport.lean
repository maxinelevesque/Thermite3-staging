/-
  Thermite/RelationalFrameTransport.lean — transport paired source results
  through the existing T1/T2 straight-line lowering theorem.

  This module intentionally transports only what `tv_meta_body` proves.  It
  does not invent a correspondence for shared-region target state; witnesses
  needing that correspondence remain `sourceOnly`.
-/

import Thermite.Faithfulness
import Thermite.RelationalFrameWitness

namespace Thermite.RelationalFrameTransport

/-- Two source executions with a derived equal result remain equal after each
    exact lowering is validated against `bodyRefState`. -/
theorem tv_meta_body_pair
    (body : Thermite.Exec.Block)
    (leftState rightState : Thermite.Exec.State)
    (leftLowered rightLowered : Option Thermite.Exec.ExecVal)
    (sourceCongruent :
      Thermite.Exec.bodyDenote body leftState =
        Thermite.Exec.bodyDenote body rightState)
    (leftTv : leftLowered = Thermite.Exec.bodyRefState body leftState)
    (rightTv : rightLowered = Thermite.Exec.bodyRefState body rightState) :
    leftLowered = rightLowered := by
  rw [Thermite.tv_meta_body body leftState leftLowered leftTv,
      Thermite.tv_meta_body body rightState rightLowered rightTv]
  exact sourceCongruent

/-! ## Exact transport for the region-free bounded return fragment -/

open Thermite.RelationalFrame.Bounded

def toExec : Thermite.RelationalFrame.Bounded.Expr → Option Thermite.Exec.ExecExpr
  | .literal (.int _) => none
  | .literal (.bool value) => some (.boolLit value)
  | .local value => some value
  | .region _ => none
  | .arith op left right => do
      some (.arith op (← toExec left) (← toExec right))
  | .compare op left right => do
      some (.cmp op (← toExec left) (← toExec right))
  | .logic op left right => do
      some (.logic op (← toExec left) (← toExec right))
  | .not value => do
      some (.not (← toExec value))
  | .cast value ty => do
      some (.cast (← toExec value) ty)

theorem eval_toExec : ∀ (expr : Thermite.RelationalFrame.Bounded.Expr)
    {encoded : Thermite.Exec.ExecExpr}
    {world : Thermite.RelationalFrame.Bounded.World},
    toExec expr = some encoded →
      expr.eval world = Thermite.Exec.execDenote encoded world.locals.env
  | .literal (.int _), encoded, _, encodedEq => by
      simp [toExec] at encodedEq
  | .literal (.bool value), encoded, _, encodedEq => by
      simp only [toExec, Option.some.injEq] at encodedEq
      subst encoded
      rfl
  | .local value, encoded, _, encodedEq => by
      simp only [toExec, Option.some.injEq] at encodedEq
      subst encoded
      rfl
  | .region _, _, _, encodedEq => by simp [toExec] at encodedEq
  | .arith op left right, encoded, world, encodedEq => by
      simp only [toExec] at encodedEq
      cases leftEq : toExec left with
      | none => simp [leftEq] at encodedEq
      | some leftEncoded =>
          cases rightEq : toExec right with
          | none => simp [leftEq, rightEq] at encodedEq
          | some rightEncoded =>
              simp [leftEq, rightEq] at encodedEq
              subst encoded
              simp only [Thermite.RelationalFrame.Bounded.Expr.eval, Thermite.Exec.execDenote]
              rw [eval_toExec left leftEq, eval_toExec right rightEq]
  | .compare op left right, encoded, world, encodedEq => by
      simp only [toExec] at encodedEq
      cases leftEq : toExec left with
      | none => simp [leftEq] at encodedEq
      | some leftEncoded =>
          cases rightEq : toExec right with
          | none => simp [leftEq, rightEq] at encodedEq
          | some rightEncoded =>
              simp [leftEq, rightEq] at encodedEq
              subst encoded
              simp only [Thermite.RelationalFrame.Bounded.Expr.eval, Thermite.Exec.execDenote]
              rw [eval_toExec left leftEq, eval_toExec right rightEq]
  | .logic op left right, encoded, world, encodedEq => by
      simp only [toExec] at encodedEq
      cases leftEq : toExec left with
      | none => simp [leftEq] at encodedEq
      | some leftEncoded =>
          cases rightEq : toExec right with
          | none => simp [leftEq, rightEq] at encodedEq
          | some rightEncoded =>
              simp [leftEq, rightEq] at encodedEq
              subst encoded
              simp only [Thermite.RelationalFrame.Bounded.Expr.eval, Thermite.Exec.execDenote]
              rw [eval_toExec left leftEq, eval_toExec right rightEq]
  | .not value, encoded, world, encodedEq => by
      simp only [toExec] at encodedEq
      cases valueEq : toExec value with
      | none => simp [valueEq] at encodedEq
      | some valueEncoded =>
          simp [valueEq] at encodedEq
          subst encoded
          simp only [Thermite.RelationalFrame.Bounded.Expr.eval, Thermite.Exec.execDenote]
          rw [eval_toExec value valueEq]
  | .cast value ty, encoded, world, encodedEq => by
      simp only [toExec] at encodedEq
      cases valueEq : toExec value with
      | none => simp [valueEq] at encodedEq
      | some valueEncoded =>
          simp [valueEq] at encodedEq
          subst encoded
          simp only [Thermite.RelationalFrame.Bounded.Expr.eval, Thermite.Exec.execDenote]
          rw [eval_toExec value valueEq]

/-- A real region-free canonical return body reaches the existing T2 target
    theorem without a caller-supplied result-congruence premise. The only
    relational input is equality of executable locals. -/
theorem bounded_return_pair_end_to_end
    (expr : Thermite.RelationalFrame.Bounded.Expr)
    (encoded : Thermite.Exec.ExecExpr)
    (encodedEq : toExec expr = some encoded)
    (left right : Thermite.RelationalFrame.Bounded.World)
    (localsEq : left.locals = right.locals) :
    Thermite.Exec.bodyRefState (.mk [] (some encoded)) left.locals =
      Thermite.Exec.bodyRefState (.mk [] (some encoded)) right.locals := by
  apply tv_meta_body_pair (.mk [] (some encoded)) left.locals right.locals
  · simp only [Thermite.Exec.bodyDenote, Thermite.Exec.Block.blkTail,
      Thermite.Exec.blockThread]
    rw [localsEq]
  · rfl
  · rfl

namespace Examples

open Thermite.Exec

def three : ExecVal := .int ⟨.u64, 3⟩

def leftEnv : ExecEnv :=
  { vars := fun name => if name = "x" then three else .bool false
    slices := fun _ => [] }

def rightEnv : ExecEnv :=
  { vars := fun name => if name = "x" then three else .bool true
    slices := fun _ => [] }

def leftState : State := ⟨leftEnv, fun _ => false⟩
def rightState : State := ⟨rightEnv, fun _ => false⟩

def identityBody : Block := .mk [] (some (.var "x"))

theorem source_pair_congruent :
    bodyDenote identityBody leftState = bodyDenote identityBody rightState := by
  simp [bodyDenote, identityBody, Block.blkTail, blockThread, execDenote,
    leftState, rightState, leftEnv, rightEnv]

/-- A concrete T2-transported pair for the compiled shape
    `fn identity(x: u64) -> u64 ! pure { x }`. -/
theorem identity_pair_end_to_end :
    bodyRefState identityBody leftState = bodyRefState identityBody rightState := by
  exact tv_meta_body_pair identityBody leftState rightState
    (bodyRefState identityBody leftState)
    (bodyRefState identityBody rightState)
    source_pair_congruent rfl rfl

end Examples

end Thermite.RelationalFrameTransport
