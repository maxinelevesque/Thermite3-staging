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
