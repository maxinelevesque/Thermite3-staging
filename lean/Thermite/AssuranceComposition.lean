import Thermite.AssurancePolicyV2

/-!
Project and portfolio composition for the V2 assurance policy.

This module deliberately does not manufacture one assurance position for a
heterogeneous portfolio.  It first proves the portfolio's item conjunction,
then computes a claim *set* in each compatible fiber.  Project aggregation
consumes one entry for every accepted population member; a failed transport is
represented by `none` and contributes `emptyNF`, never by dropping the member.
-/

namespace Thermite.CertificationMetatheory

open Thermite.LanguageCompleteness

/-! ## Exact source/build population -/

structure ProjectBuildIdentity where
  crateName : String
  target : String
  features : List String
  platform : String
  generatedSources : List String
  artifactSha256 : String
deriving DecidableEq, Repr

structure ProjectItemIdentity where
  sourcePath : String
  itemPath : String
deriving DecidableEq, Repr

inductive ProjectDisposition where
  | accepted
  | nonClaim (reason : String)
  | legacyUnversioned (legacyLevel : String)
deriving DecidableEq, Repr

structure ProjectPopulationMember where
  identity : ProjectItemIdentity
  disposition : ProjectDisposition
deriving DecidableEq, Repr

/-- The constructor proof makes the source-ordered population a total exact
partition: every intended item occurs once, in the same order, with exactly one
closed disposition. -/
structure ProjectPopulation where
  build : ProjectBuildIdentity
  intended : List ProjectItemIdentity
  members : List ProjectPopulationMember
  intendedNodup : intended.Nodup
  totalPartition : members.map ProjectPopulationMember.identity = intended

def ProjectPopulation.acceptedIdentities (population : ProjectPopulation) :
    List ProjectItemIdentity :=
  (population.members.filter fun member =>
    match member.disposition with
    | .accepted => true
    | _ => false).map ProjectPopulationMember.identity

def ProjectPopulation.excludedIdentities (population : ProjectPopulation) :
    List ProjectItemIdentity :=
  (population.members.filter fun member =>
    match member.disposition with
    | .accepted => false
    | _ => true).map ProjectPopulationMember.identity

def ProjectPopulation.allAccepted (population : ProjectPopulation) : Bool :=
  population.members.all fun member =>
    match member.disposition with
    | .accepted => true
    | _ => false

theorem ProjectPopulation.member_identities_are_exact
    (population : ProjectPopulation) :
    population.members.map ProjectPopulationMember.identity = population.intended :=
  population.totalPartition

theorem ProjectPopulation.member_identities_are_nodup
    (population : ProjectPopulation) :
    (population.members.map ProjectPopulationMember.identity).Nodup := by
  rw [population.totalPartition]
  exact population.intendedNodup

/-! ## Clause portfolios and item conjunctions -/

/-- A clause whose accepted evidence already has a sound semantic meaning. -/
structure AddressedClauseClaim where
  address : String
  claim : Program → Prop
  evidenceIdentity : String
  accepts : Program → Prop
  evidence : ∀ program, accepts program → claim program

/-- `PortfolioLift` is the only admission path for a heterogeneous item.  Exact
list equality simultaneously pins source order, completeness, and absence of
duplicates (because the expected inventory is proved duplicate-free). -/
structure PortfolioLift where
  item : ProjectItemIdentity
  expectedClauses : List String
  expectedClausesNodup : expectedClauses.Nodup
  expectedClausesNonempty : expectedClauses ≠ []
  clauses : List AddressedClauseClaim
  complete : clauses.map AddressedClauseClaim.address = expectedClauses
  itemClaim : Program → Prop
  connective : ∀ program,
    (∀ clause, clause ∈ clauses → clause.claim program) → itemClaim program

def PortfolioLift.evidenceVector (lift : PortfolioLift) : List String :=
  lift.clauses.map AddressedClauseClaim.evidenceIdentity

theorem PortfolioLift.certifiesItem (lift : PortfolioLift) (program : Program)
    (covered : ∀ clause, clause ∈ lift.clauses → clause.accepts program) :
    lift.itemClaim program := by
  apply lift.connective program
  intro clause member
  exact clause.evidence program (covered clause member)

theorem PortfolioLift.addressesAreExact (lift : PortfolioLift) :
    lift.clauses.map AddressedClauseClaim.address = lift.expectedClauses :=
  lift.complete

theorem PortfolioLift.noClauseCanBeDropped (lift : PortfolioLift) :
    lift.clauses.length = lift.expectedClauses.length := by
  have same := congrArg List.length lift.complete
  simpa using same

/-! ## Item claim sets and project frontiers -/

def AssuranceKindSet.union (left right : AssuranceKindSet) : AssuranceKindSet :=
  AssuranceKindSet.ofPredicate fun kind => left.contains kind || right.contains kind

@[simp] theorem AssuranceKindSet.contains_union
    (left right : AssuranceKindSet) (kind : AssuranceKindV2) :
    (left.union right).contains kind =
      (left.contains kind || right.contains kind) := by
  simp [AssuranceKindSet.union]

def unionNF {fiber} (left right : AntichainNF fiber) : AntichainNF fiber where
  support := left.support.union right.support
  allowed := by
    intro kind member
    rw [AssuranceKindSet.contains_union] at member
    rcases Bool.or_eq_true_iff.mp member with leftMember | rightMember
    · exact left.allowed kind leftMember
    · exact right.allowed kind rightMember
  closed := by
    intro lower upper lowerAllowed upperAllowed lowerUpper upperMember
    rw [AssuranceKindSet.contains_union] at upperMember
    rw [AssuranceKindSet.contains_union]
    rcases Bool.or_eq_true_iff.mp upperMember with leftMember | rightMember
    · exact Bool.or_eq_true_iff.mpr <| Or.inl <|
        left.closed lower upper lowerAllowed upperAllowed lowerUpper leftMember
    · exact Bool.or_eq_true_iff.mpr <| Or.inr <|
        right.closed lower upper lowerAllowed upperAllowed lowerUpper rightMember

theorem unionNF_denotation {fiber} (left right : AntichainNF fiber)
    (point : AssurancePolicyV2) :
    (unionNF left right).denotes point ↔
      left.denotes point ∨ right.denotes point := by
  rcases point with ⟨pointFiber, pointKind⟩
  change pointFiber = fiber ∧
      (left.support.union right.support).contains pointKind = true ↔ _
  rw [AssuranceKindSet.contains_union]
  rw [Bool.or_eq_true_iff]
  simp only [AntichainNF.denotes]
  change pointFiber = fiber ∧
      (left.support.contains pointKind = true ∨
        right.support.contains pointKind = true) ↔ _
  constructor
  · rintro ⟨sameFiber, leftMember | rightMember⟩
    · exact Or.inl ⟨sameFiber, leftMember⟩
    · exact Or.inr ⟨sameFiber, rightMember⟩
  · rintro (⟨sameFiber, leftMember⟩ | ⟨sameFiber, rightMember⟩)
    · exact ⟨sameFiber, Or.inl leftMember⟩
    · exact ⟨sameFiber, Or.inr rightMember⟩

def unionAllNF {fiber} : List (AntichainNF fiber) → AntichainNF fiber
  | [] => emptyNF fiber
  | first :: rest => rest.foldl unionNF first

structure ClauseClaimSetEntry (fiber : ClaimFiberKeyV2) where
  address : String
  transported : Option (AntichainNF fiber)

def ClauseClaimSetEntry.claimSet {fiber} (entry : ClauseClaimSetEntry fiber) :
    AntichainNF fiber :=
  entry.transported.getD (emptyNF fiber)

/-- A heterogeneous item's set is constructible only from a complete
`PortfolioLift` and one source-ordered transport cell per lifted clause. -/
structure HeterogeneousItemClaimSetInput (fiber : ClaimFiberKeyV2)
    (lift : PortfolioLift) where
  clauses : List (ClauseClaimSetEntry fiber)
  complete : clauses.map ClauseClaimSetEntry.address = lift.expectedClauses

def intersectClaimSets {fiber} : List (AntichainNF fiber) → AntichainNF fiber
  | [] => emptyNF fiber
  | first :: rest => intersectNonemptyNF first rest

theorem intersectClaimSets_member {fiber} (sets : List (AntichainNF fiber))
    (normal : AntichainNF fiber) (normalMember : normal ∈ sets)
    (kind : AssuranceKindV2)
    (supported : (intersectClaimSets sets).support.contains kind = true) :
    normal.support.contains kind = true := by
  cases sets with
  | nil => simp at normalMember
  | cons first rest =>
      have denoted : (intersectNonemptyNF first rest).denotes ⟨fiber, kind⟩ :=
        ⟨rfl, supported⟩
      have every := (intersectNonemptyNF_denotation first rest
        ⟨fiber, kind⟩).mp denoted
      simp only [List.mem_cons] at normalMember
      rcases normalMember with equal | restMember
      · subst normal
        exact every.1.2
      · exact (every.2 normal restMember).2

theorem intersectClaimSets_denotation {fiber}
    (sets : List (AntichainNF fiber)) (nonempty : sets ≠ [])
    (point : AssurancePolicyV2) :
    (intersectClaimSets sets).denotes point ↔
      ∀ normal, normal ∈ sets → normal.denotes point := by
  cases sets with
  | nil => contradiction
  | cons first rest =>
      rw [intersectClaimSets, intersectNonemptyNF_denotation]
      constructor
      · rintro ⟨firstDenotes, restDenotes⟩ normal member
        simp only [List.mem_cons] at member
        rcases member with equal | restMember
        · subst normal
          exact firstDenotes
        · exact restDenotes normal restMember
      · intro allDenote
        exact ⟨allDenote first (by simp), fun normal member =>
          allDenote normal (by simp [member])⟩

theorem intersectClaimSets_permutation_invariant {fiber}
    {left right : List (AntichainNF fiber)} (leftNonempty : left ≠ [])
    (permutation : left.Perm right) :
    intersectClaimSets left = intersectClaimSets right := by
  have rightNonempty : right ≠ [] := by
    intro empty
    rw [empty] at permutation
    have := permutation.length_eq
    simp at this
    exact leftNonempty this
  apply antichain_nf_is_extensional
  intro kind
  have semantic :
      (intersectClaimSets left).denotes ⟨fiber, kind⟩ ↔
        (intersectClaimSets right).denotes ⟨fiber, kind⟩ := by
    rw [intersectClaimSets_denotation left leftNonempty,
      intersectClaimSets_denotation right rightNonempty]
    constructor
    · intro allDenote normal member
      exact allDenote normal (permutation.mem_iff.mpr member)
    · intro allDenote normal member
      exact allDenote normal (permutation.mem_iff.mp member)
  cases leftMember : (intersectClaimSets left).support.contains kind <;>
    cases rightMember : (intersectClaimSets right).support.contains kind <;>
    simp_all [AntichainNF.denotes]

theorem intersectClaimSets_duplicate_invariant {fiber}
    (normal : AntichainNF fiber) (rest : List (AntichainNF fiber)) :
    intersectClaimSets (normal :: normal :: rest) =
      intersectClaimSets (normal :: rest) := by
  apply antichain_nf_is_extensional
  intro kind
  have semantic :
      (intersectClaimSets (normal :: normal :: rest)).denotes ⟨fiber, kind⟩ ↔
        (intersectClaimSets (normal :: rest)).denotes ⟨fiber, kind⟩ := by
    rw [intersectClaimSets_denotation (normal :: normal :: rest) (by simp),
      intersectClaimSets_denotation (normal :: rest) (by simp)]
    simp
  cases leftMember :
      (intersectClaimSets (normal :: normal :: rest)).support.contains kind <;>
    cases rightMember :
      (intersectClaimSets (normal :: rest)).support.contains kind <;>
    simp_all [AntichainNF.denotes]

structure ItemClaimSet (fiber : ClaimFiberKeyV2) where
  item : ProjectItemIdentity
  normal : AntichainNF fiber
  clauseProvenance : List String

def homogeneousItemClaimSet (item : ProjectItemIdentity)
    (point : AssurancePolicyV2) : ItemClaimSet point.fiber :=
  ⟨item, principalNF point, []⟩

def heterogeneousItemClaimSet {fiber} (lift : PortfolioLift)
    (input : HeterogeneousItemClaimSetInput fiber lift) : ItemClaimSet fiber :=
  ⟨lift.item, intersectClaimSets (input.clauses.map ClauseClaimSetEntry.claimSet),
    lift.expectedClauses⟩

theorem heterogeneous_item_uses_every_clause {fiber} (lift : PortfolioLift)
    (input : HeterogeneousItemClaimSetInput fiber lift) :
    input.clauses.length = lift.expectedClauses.length := by
  have same := congrArg List.length input.complete
  simpa using same

theorem unavailable_clause_makes_item_claim_set_empty {fiber}
    (lift : PortfolioLift) (input : HeterogeneousItemClaimSetInput fiber lift)
    (entry : ClauseClaimSetEntry fiber) (entryMember : entry ∈ input.clauses)
    (unavailable : entry.transported = none) :
    (heterogeneousItemClaimSet lift input).normal = emptyNF fiber := by
  apply AntichainNF.ext
  apply AssuranceKindSet.ext
  intro kind
  rw [show (emptyNF fiber).support.contains kind = false by
    cases kind <;> rfl]
  simp only [heterogeneousItemClaimSet]
  have mapped : entry.claimSet ∈ input.clauses.map ClauseClaimSetEntry.claimSet :=
    List.mem_map.mpr ⟨entry, entryMember, rfl⟩
  have entryEmpty : entry.claimSet = emptyNF fiber := by
    simp [ClauseClaimSetEntry.claimSet, unavailable]
  cases supported : (intersectClaimSets
      (input.clauses.map ClauseClaimSetEntry.claimSet)).support.contains kind with
  | false => rfl
  | true =>
      have entrySupported := intersectClaimSets_member
        (input.clauses.map ClauseClaimSetEntry.claimSet) entry.claimSet mapped kind supported
      rw [entryEmpty] at entrySupported
      cases kind <;>
        simp [emptyNF, AssuranceKindSet.empty, AssuranceKindSet.contains] at entrySupported

structure PopulationClaimSetEntry (fiber : ClaimFiberKeyV2) where
  item : ProjectItemIdentity
  /-- `none` is an explicit failed/incompatible transport and denotes empty. -/
  transported : Option (ItemClaimSet fiber)

def PopulationClaimSetEntry.claimSet {fiber}
    (entry : PopulationClaimSetEntry fiber) : AntichainNF fiber :=
  match entry.transported with
  | some claimSet => claimSet.normal
  | none => emptyNF fiber

/-- Exact project aggregation input: the entry identities must equal the full
accepted population in source order. -/
structure ProjectClaimSetInput (population : ProjectPopulation)
    (fiber : ClaimFiberKeyV2) where
  entries : List (PopulationClaimSetEntry fiber)
  complete : entries.map PopulationClaimSetEntry.item = population.acceptedIdentities

def ProjectClaimSetInput.commonClaimSet {population fiber}
    (input : ProjectClaimSetInput population fiber) : FiniteIntersectionV2 fiber :=
  intersectAllNF (input.entries.map PopulationClaimSetEntry.claimSet)

def ProjectClaimSetInput.evidenceClaimSet {population fiber}
    (input : ProjectClaimSetInput population fiber) : AntichainNF fiber :=
  unionAllNF (input.entries.map PopulationClaimSetEntry.claimSet)

def ProjectClaimSetInput.commonClaimFrontier {population fiber}
    (input : ProjectClaimSetInput population fiber) : Option (List AssuranceKindV2) :=
  match input.commonClaimSet with
  | .noItems => none
  | .common normal => some normal.frontier

structure EvidenceFrontierEntry where
  kind : AssuranceKindV2
  supporters : List (ProjectItemIdentity × List String)
deriving DecidableEq, Repr

def ProjectClaimSetInput.evidenceFrontier {population fiber}
    (input : ProjectClaimSetInput population fiber) : List EvidenceFrontierEntry :=
  input.evidenceClaimSet.frontier.map fun kind =>
    ⟨kind, input.entries.filterMap fun entry =>
      match entry.transported with
      | some claimSet =>
          if claimSet.normal.support.contains kind then
            some (entry.item, claimSet.clauseProvenance)
          else none
      | none => none⟩

theorem project_input_cannot_omit_accepted_member {population fiber}
    (input : ProjectClaimSetInput population fiber) :
    input.entries.length = population.acceptedIdentities.length := by
  have same := congrArg List.length input.complete
  simpa using same

theorem explicit_absent_transport_makes_common_empty {population fiber}
    (input : ProjectClaimSetInput population fiber)
    (entry : PopulationClaimSetEntry fiber) (member : entry ∈ input.entries)
    (absent : entry.transported = none) :
    input.commonClaimSet = .common (emptyNF fiber) := by
  unfold ProjectClaimSetInput.commonClaimSet
  have mapped : entry.claimSet ∈ input.entries.map PopulationClaimSetEntry.claimSet :=
    List.mem_map.mpr ⟨entry, member, rfl⟩
  have nonempty : input.entries.map PopulationClaimSetEntry.claimSet ≠ [] := by
    intro empty
    rw [empty] at mapped
    simp at mapped
  cases sets : input.entries.map PopulationClaimSetEntry.claimSet with
  | nil => exact False.elim (nonempty sets)
  | cons first rest =>
      simp only [intersectAllNF]
      congr 1
      apply AntichainNF.ext
      apply AssuranceKindSet.ext
      intro kind
      rw [show (emptyNF fiber).support.contains kind = false by cases kind <;> rfl]
      have absentEmpty : entry.claimSet = emptyNF fiber := by
        simp [PopulationClaimSetEntry.claimSet, absent]
      have mapped' : entry.claimSet = first ∨ entry.claimSet ∈ rest := by
        simpa [sets] using mapped
      cases supported : (intersectNonemptyNF first rest).support.contains kind with
      | false => rfl
      | true =>
          have denoted : (intersectNonemptyNF first rest).denotes ⟨fiber, kind⟩ :=
            ⟨rfl, supported⟩
          have every := (intersectNonemptyNF_denotation first rest
            ⟨fiber, kind⟩).mp denoted
          have entrySupported : entry.claimSet.support.contains kind = true := by
            rcases mapped' with equal | restMember
            · rw [equal]
              exact every.1.2
            · exact (every.2 entry.claimSet restMember).2
          rw [absentEmpty] at entrySupported
          cases kind <;>
            simp [emptyNF, AssuranceKindSet.empty,
              AssuranceKindSet.contains] at entrySupported

inductive ProjectPortraitScope where
  /-- A complete population intersection is a common policy floor, not a
  realized whole-project procedure result. -/
  | completePopulationFloor (members : Nat)
  | wholeProject
  | acceptedSubset (numerator denominator : Nat)
      (excluded : List ProjectItemIdentity)
deriving DecidableEq, Repr

def ProjectPopulation.portraitScope (population : ProjectPopulation) :
    ProjectPortraitScope :=
  if population.allAccepted then .completePopulationFloor population.intended.length
  else .acceptedSubset population.acceptedIdentities.length
    population.intended.length population.excludedIdentities

theorem ProjectPopulation.portraitScope_is_never_whole
    (population : ProjectPopulation) :
    population.portraitScope ≠ .wholeProject := by
  cases accepted : population.allAccepted <;>
    simp [ProjectPopulation.portraitScope, accepted]

theorem nonaccepted_member_suppresses_whole_project
    (population : ProjectPopulation) (notAll : population.allAccepted = false) :
    population.portraitScope = .acceptedSubset population.acceptedIdentities.length
      population.intended.length population.excludedIdentities := by
  simp [ProjectPopulation.portraitScope, notAll]

/-! ## Project judgment and refutation -/

structure LiftedItemClaim where
  identity : ProjectItemIdentity
  claim : Program → Prop
  evidenceIdentity : String
  accepts : Program → Prop
  evidence : ∀ program, accepts program → claim program
  observation : Program → Prop
  refutationSound : ∀ program, observation program → ¬claim program

/-- `ProjectLift` is available only for an all-accepted population and an exact
source-ordered item-evidence vector. -/
structure ProjectLift (population : ProjectPopulation) where
  /-- The exact candidate claim fiber and complete aggregation input whose
  item transports this lift realizes. -/
  transportFiber : ClaimFiberKeyV2
  transportInput : ProjectClaimSetInput population transportFiber
  /-- A whole-project result cannot be manufactured from an unavailable item
  transport, even though an unavailable transport is still represented as
  empty in the reportable common floor. -/
  allTransported : ∀ entry, entry ∈ transportInput.entries →
    ∃ claimSet, entry.transported = some claimSet
  /-- `NoItems` is reportable, but an empty project cannot acquire a vacuous
  whole-project certification. -/
  nonempty : population.intended ≠ []
  allAccepted : population.allAccepted = true
  items : List LiftedItemClaim
  complete : items.map LiftedItemClaim.identity = population.intended

/-- Only a realized `ProjectLift`, never aggregation alone, authorizes the
whole-project headline. -/
def ProjectLift.portraitScope {population} (_ : ProjectLift population) :
    ProjectPortraitScope := .wholeProject

theorem ProjectLift.authorizes_whole_project {population}
    (lift : ProjectLift population) :
    lift.portraitScope = .wholeProject := rfl

theorem ProjectLift.every_item_transport_present {population}
    (lift : ProjectLift population) (entry : PopulationClaimSetEntry lift.transportFiber)
    (member : entry ∈ lift.transportInput.entries) :
    ∃ claimSet, entry.transported = some claimSet :=
  lift.allTransported entry member

def ProjectLift.evidenceVector {population} (lift : ProjectLift population) :
    List String := lift.items.map LiftedItemClaim.evidenceIdentity

def ProjectLift.projectClaim {population} (lift : ProjectLift population)
    (program : Program) : Prop :=
  ∀ item, item ∈ lift.items → item.claim program

def ProjectLift.projectPopulation {population} (lift : ProjectLift population)
    (program : Program) : Prop :=
  ∀ item, item ∈ lift.items → item.accepts program

structure ProjectEvidenceVector {population} (lift : ProjectLift population) where
  identities : List String
  sourceOrdered : identities = lift.evidenceVector

def ProjectJudgment {population} (lift : ProjectLift population)
    (evidence : ProjectEvidenceVector lift) (program : Program) : Prop :=
  evidence.identities = lift.evidenceVector ∧
    lift.projectPopulation program ∧ lift.projectClaim program

def ProjectLift.realizedEvidence {population} (lift : ProjectLift population) :
    ProjectEvidenceVector lift := ⟨lift.evidenceVector, rfl⟩

theorem ProjectLift.certifiesProject {population} (lift : ProjectLift population)
    (program : Program) (inside : lift.projectPopulation program) :
    ProjectJudgment lift lift.realizedEvidence program := by
  refine ⟨rfl, inside, ?_⟩
  intro item member
  exact item.evidence program (inside item member)

def ProjectLift.observesFalseConjunct {population} (lift : ProjectLift population)
    (program : Program) : Prop :=
  ∃ item, item ∈ lift.items ∧ item.observation program

theorem ProjectLift.falseConjunctRefutes {population} (lift : ProjectLift population)
    {program : Program} (observed : lift.observesFalseConjunct program) :
    ¬lift.projectClaim program := by
  rintro projectClaim
  rcases observed with ⟨item, member, itemObservation⟩
  exact item.refutationSound program itemObservation (projectClaim item member)

/-- Completeness requires both a total finite scheduler for a false conjunct
and a completeness premise for every item; neither is inferred from soundness. -/
structure ProjectRefutationCompleteness {population} (lift : ProjectLift population) where
  schedule : ∀ program, ¬lift.projectClaim program → LiftedItemClaim
  scheduledMember : ∀ program (notProject : ¬lift.projectClaim program),
    schedule program notProject ∈ lift.items
  scheduledFalse : ∀ program (notProject : ¬lift.projectClaim program),
    ¬(schedule program notProject).claim program
  itemComplete : ∀ item, item ∈ lift.items → ∀ program,
    ¬item.claim program → item.observation program

theorem ProjectRefutationCompleteness.complete {population}
    {lift : ProjectLift population} (premises : ProjectRefutationCompleteness lift)
    {program : Program} (notProject : ¬lift.projectClaim program) :
    lift.observesFalseConjunct program := by
  let item := premises.schedule program notProject
  exact ⟨item, premises.scheduledMember program notProject,
    premises.itemComplete item (premises.scheduledMember program notProject)
      program (premises.scheduledFalse program notProject)⟩

/-! ## Hostile fixtures -/

def fixtureBuildIdentity : ProjectBuildIdentity :=
  ⟨"fixture", "lib", ["default"], "x86_64-linux", ["generated.rs"], "artifact"⟩

def fixtureItemA : ProjectItemIdentity := ⟨"src/lib.th", "crate::a"⟩
def fixtureItemB : ProjectItemIdentity := ⟨"src/lib.th", "crate::b"⟩

def fixturePopulation : ProjectPopulation where
  build := fixtureBuildIdentity
  intended := [fixtureItemA, fixtureItemB]
  members := [⟨fixtureItemA, .accepted⟩, ⟨fixtureItemB, .accepted⟩]
  intendedNodup := by decide
  totalPartition := rfl

def solverItemSet : ItemClaimSet allInputFixtureFiber :=
  homogeneousItemClaimSet fixtureItemA ⟨allInputFixtureFiber, .solverComplete⟩

def empiricalItemSet : ItemClaimSet allInputFixtureFiber :=
  homogeneousItemClaimSet fixtureItemB ⟨allInputFixtureFiber, .leanEmpirical⟩

def fixtureClause (address evidenceIdentity : String) : AddressedClauseClaim where
  address := address
  claim := fun _ => True
  evidenceIdentity := evidenceIdentity
  accepts := fun _ => True
  evidence := by simp

def mixedPortfolioLift : PortfolioLift where
  item := fixtureItemA
  expectedClauses := ["requires[0]", "ensures[0]"]
  expectedClausesNodup := by decide
  expectedClausesNonempty := by decide
  clauses := [fixtureClause "requires[0]" "solver-evidence",
    fixtureClause "ensures[0]" "lean-evidence"]
  complete := rfl
  itemClaim := fun _ => True
  connective := by simp

def mixedPortfolioClaimSetInput :
    HeterogeneousItemClaimSetInput allInputFixtureFiber mixedPortfolioLift where
  clauses := [⟨"requires[0]", some solverCompleteNF⟩,
    ⟨"ensures[0]", some empiricalLeanNF⟩]
  complete := rfl

theorem mixed_portfolio_lift_certifies_item (program : Program) :
    mixedPortfolioLift.itemClaim program := by
  apply mixedPortfolioLift.certifiesItem program
  intro clause member
  simp only [mixedPortfolioLift, List.mem_cons] at member
  rcases member with equal | member
  · subst clause
    trivial
  · rcases member with equal | impossible
    · subst clause
      trivial
    · simp at impossible

theorem mixed_portfolio_claim_set_is_exact_intersection :
    (heterogeneousItemClaimSet mixedPortfolioLift
      mixedPortfolioClaimSetInput).normal.frontier = [.solverIncomplete] := by decide

theorem dropped_clause_cannot_satisfy_complete_portfolio
    (lift : PortfolioLift)
    (expected : lift.expectedClauses = ["requires[0]", "ensures[0]"])
    (dropped : lift.clauses = [fixtureClause "requires[0]" "solver-evidence"]) :
    False := by
  have lengths := lift.noClauseCanBeDropped
  rw [expected, dropped] at lengths
  contradiction

def mixedProjectInput : ProjectClaimSetInput fixturePopulation allInputFixtureFiber where
  entries := [⟨fixtureItemA, some solverItemSet⟩,
    ⟨fixtureItemB, some empiricalItemSet⟩]
  complete := rfl

theorem mixed_incomparable_common_frontier_is_not_a_representative :
    mixedProjectInput.commonClaimFrontier = some [.solverIncomplete] ∧
    mixedProjectInput.commonClaimFrontier ≠ some [.solverComplete] ∧
    mixedProjectInput.commonClaimFrontier ≠ some [.leanEmpirical] := by decide

theorem mixed_incomparable_evidence_frontier_retains_provenance :
    mixedProjectInput.evidenceFrontier =
      [⟨.solverComplete, [(fixtureItemA, [])]⟩,
        ⟨.leanEmpirical, [(fixtureItemB, [])]⟩] := by decide

theorem per_axis_or_representative_strength_cannot_replace_common_claim :
    (intersectNF solverCompleteNF empiricalLeanNF) ≠ solverCompleteNF ∧
      (intersectNF solverCompleteNF empiricalLeanNF) ≠ empiricalLeanNF := by
  constructor
  · intro equal
    have frontiers := congrArg AntichainNF.frontier equal
    have intersection : (intersectNF solverCompleteNF empiricalLeanNF).frontier =
        [.solverIncomplete] := incomparable_intersection_fixture
    have solver : solverCompleteNF.frontier = [.solverComplete] :=
      singleton_frontier_fixture
    rw [intersection, solver] at frontiers
    contradiction
  · intro equal
    have frontiers := congrArg AntichainNF.frontier equal
    have intersection : (intersectNF solverCompleteNF empiricalLeanNF).frontier =
        [.solverIncomplete] := incomparable_intersection_fixture
    have empirical : empiricalLeanNF.frontier = [.leanEmpirical] := by decide
    rw [intersection, empirical] at frontiers
    contradiction

def subsetFixturePopulation : ProjectPopulation where
  build := fixtureBuildIdentity
  intended := [fixtureItemA, fixtureItemB]
  members := [⟨fixtureItemA, .accepted⟩,
    ⟨fixtureItemB, .legacyUnversioned "L3"⟩]
  intendedNodup := by decide
  totalPartition := rfl

theorem legacy_member_yields_exact_subset_portrait :
    subsetFixturePopulation.portraitScope =
      .acceptedSubset 1 2 [fixtureItemB] := by decide

def trueFixtureItem (identity : ProjectItemIdentity) : LiftedItemClaim where
  identity := identity
  claim := fun _ => True
  evidenceIdentity := identity.itemPath ++ ":evidence"
  accepts := fun _ => True
  evidence := by simp
  observation := fun _ => False
  refutationSound := by simp

def fixtureProjectLift : ProjectLift fixturePopulation where
  transportFiber := allInputFixtureFiber
  transportInput := mixedProjectInput
  allTransported := by
    intro entry member
    simp only [mixedProjectInput, List.mem_cons] at member
    rcases member with equal | member
    · subst entry
      exact ⟨solverItemSet, rfl⟩
    · rcases member with equal | impossible
      · subst entry
        exact ⟨empiricalItemSet, rfl⟩
      · simp at impossible
  nonempty := by decide
  allAccepted := by decide
  items := [trueFixtureItem fixtureItemA, trueFixtureItem fixtureItemB]
  complete := rfl

theorem multi_item_project_lift_transports_every_item
    (program : Program) :
    ProjectJudgment fixtureProjectLift fixtureProjectLift.realizedEvidence program := by
  exact fixtureProjectLift.certifiesProject program (by
    intro item member
    simp only [fixtureProjectLift, List.mem_cons] at member
    rcases member with equal | member
    · subst item
      trivial
    · rcases member with equal | impossible
      · subst item
        trivial
      · simp at impossible)

theorem project_lift_evidence_vector_is_source_ordered :
    fixtureProjectLift.evidenceVector =
      [fixtureItemA.itemPath ++ ":evidence", fixtureItemB.itemPath ++ ":evidence"] := rfl

def singleFixturePopulation : ProjectPopulation where
  build := fixtureBuildIdentity
  intended := [fixtureItemA]
  members := [⟨fixtureItemA, .accepted⟩]
  intendedNodup := by decide
  totalPartition := rfl

def falseFixtureItem : LiftedItemClaim where
  identity := fixtureItemA
  claim := fun _ => False
  evidenceIdentity := "false-item-evidence"
  accepts := fun _ => False
  evidence := by simp
  observation := fun _ => True
  refutationSound := by simp

def singleProjectInput :
    ProjectClaimSetInput singleFixturePopulation allInputFixtureFiber where
  entries := [⟨fixtureItemA, some solverItemSet⟩]
  complete := rfl

def falseFixtureProjectLift : ProjectLift singleFixturePopulation where
  transportFiber := allInputFixtureFiber
  transportInput := singleProjectInput
  allTransported := by
    intro entry member
    have equal : entry = ⟨fixtureItemA, some solverItemSet⟩ := by
      simpa [singleProjectInput] using member
    subst entry
    exact ⟨solverItemSet, rfl⟩
  nonempty := by decide
  allAccepted := by decide
  items := [falseFixtureItem]
  complete := rfl

theorem one_false_conjunct_soundly_refutes_project (program : Program) :
    ¬falseFixtureProjectLift.projectClaim program := by
  apply falseFixtureProjectLift.falseConjunctRefutes
  exact ⟨falseFixtureItem, by simp [falseFixtureProjectLift], trivial⟩

end Thermite.CertificationMetatheory
