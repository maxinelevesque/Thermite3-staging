---
{
  "v": 2,
  "cid": "bafyreifvhri44qr5mokst2o7wrpxjmbau7s3xsfpiei4xhepwwpc4lazym",
  "sig": "1cd3c605dbfe1191d65a021ddcc45cc591c7b36a2d2735baa2a14d7edbb32bfc399dfa1171052618fc7e460b6a76d5fe830f4f6a767020a1ccd9a1fa901fe4f1",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/19-proof-layer\")",
  "kind": "Observation",
  "cites": [],
  "rev": "223msswc6un7r",
  "seq": 0,
  "of": 12,
  "text_len": 655,
  "content": "a764626f6479a16b4f62736572766174696f6ea16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c727266632f31392d70726f6f662d6c617965726961727469666163747382a166436f6d6d6974782862373962343030353833643830666565373133383962666334626633333135303737363763396265a16646696c6541748278242e64657369676e2f726663732f303031392d7468652d70726f6f662d6c617965722e6d6478286237396234303035383364383066656537313338396266633462663333313530373736376339626569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658c7104d4c3d"
}
---

The proof/program seam is already built three times under three names and nobody has named it as one construct. (a) RFC-7 §6's 'opaque spec fn' seals a spec body behind its ensures. (b) stage1-forge-tier REQ-3's Item::Forge — prop fn / lemma / proof for f / witness — is out-of-line proof material bound by semantic address, and it parses TODAY (thermite-syntax/src/ast.rs ForgeItem, address.rs AddrKind::Forge). (c) forge review (SHIPPED, forge/src/review.rs) projects the contract + spec-fn declarations and structurally never reads body. So the compiler already computes the boundary; what is missing is a source form that lets an author write it.
---8<---
---
{
  "v": 2,
  "cid": "bafyreie3ztphkv3jn7glnjp2soxgxcramvqr6npvrq3sd5fan2a3iihaam",
  "sig": "3b0a30a7715abb7e9f2a03d0d68fb288c2a744f0d28dfe49cda859fef09763675933cf98182d96d20c92d51ec9425eb44c5474fb21a158ed2a9fbcda826f9218",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/19-proof-layer\")",
  "kind": "Subject",
  "cites": [],
  "rev": "223msswc6xgq7",
  "seq": 1,
  "of": 12,
  "content": "a764626f6479a1675375626a656374a2657469746c65775246432d31393a207468652070726f6f66206c617965726c7375626a6563745f6b696e6464496465616563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c727266632f31392d70726f6f662d6c617965726961727469666163747381a166436f6d6d697478286237396234303035383364383066656537313338396266633462663333313530373736376339626569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658c7104eb252"
}
---
---8<---
---
{
  "v": 2,
  "cid": "bafyreib7fpigg6otslqtgnoajgwxpwc4t6ul75re37eoxodtws3emj55si",
  "sig": "18032f9fe871828a84880fedc0f8fb8fa7feb92f91dd97c44a0de1620062bc347ec1ba805c8e687a9bcb74bc953d221766713bba7946dd87daa28414e8f14f7b",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/19-proof-layer\")",
  "kind": "Decision",
  "cites": [
    "bafyreifvhri44qr5mokst2o7wrpxjmbau7s3xsfpiei4xhepwwpc4lazym"
  ],
  "rev": "223msswcii4gv",
  "seq": 2,
  "of": 12,
  "text_len": 642,
  "content": "a764626f6479a1684465636973696f6ea164746578746065636974657381d82a58250001711220b53c51ce423d639529e9dfb45f74b020a7e5bbc8af4111cb9c8fb59e2e2c19c366617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c727266632f31392d70726f6f662d6c617965726961727469666163747381a166436f6d6d697478286237396234303035383364383066656537313338396266633462663333313530373736376339626569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658c710e70924"
}
---

The proof layer takes ITEM-keyed material and leaves BODY-keyed material inline, and the deciding fact is address stability rather than taste. semantic-addressing.md REQ-2 numbers loops in source order within a function, and REQ-5 scopes stability to unrelated-item edits, loop-body-statement edits and renames — so ADDING a loop renumbers every later loop. An out-of-line f.loop#2.keeps#1 silently retargets when an author inserts a loop above it. Contract clauses renumber too, but only under an edit to the contract, which is the layer the hint belongs to. Hence: 'by' hints, lemma, proof for, witness move out; loop keeps/measures stay.
---8<---
---
{
  "v": 2,
  "cid": "bafyreicugxwrdjwygrru7bc5srbb6ibkvtuz6vupsaq3rzziql2earp73y",
  "sig": "a663464ca311d9c5dd5fe1b8ea2625212dd148a6fc9a72e4c72a60eeed8527710113a630d13f0f93b66d6d1d4661a59593bcce6e2ff88f39cde3697133aaf2a1",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/19-proof-layer\")",
  "kind": "Observation",
  "cites": [
    "bafyreib7fpigg6otslqtgnoajgwxpwc4t6ul75re37eoxodtws3emj55si"
  ],
  "rev": "223msswcthx5r",
  "seq": 3,
  "of": 12,
  "text_len": 843,
  "content": "a764626f6479a16b4f62736572766174696f6ea164746578746065636974657381d82a582500017112203f2bd06379d392e13335c049ad77d85c9fa8bff624dfc8ebb873b4b64627bd9266617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c727266632f31392d70726f6f662d6c617965726961727469666163747381a166436f6d6d697478286237396234303035383364383066656537313338396266633462663333313530373736376339626569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658c71196f405"
}
---

Two claims that look like payoffs of the proof layer and are not, checked and rejected during the design pass. (1) It does NOT fix RFC-7's filed diverge-loop defect ('a loop with fx diverge on the enclosing fn and no measure' -> missing mandatory dec). That defect is loop-local; the fn-level diverge exemption already works per RFC-7's own probe table. The fix is extending the exemption to loops inside a diverge fn, independent of any layering. (2) 'separate the specification from the implementation' is a DIFFERENT move from 'separate proof from program' — it binds by a named abstract model rather than by semantic address, and its obligation is refinement rather than discharge. It is blocked on there being no module system: surface-grammar.md REQ-1 admits three top-level item forms and lists mod/impl/trait as having no production.
---8<---
---
{
  "v": 2,
  "cid": "bafyreieuziqff2vx3ns7nehoqvesaexmkqz2qugi2o3ffpyno34koigncu",
  "sig": "d950e2591ecb47135b243d4fb480dd3c5c181afc053180ad5817843262aeadac0f4a576502ad4d58e2518451ff9836729dfb707c429fa02cc0d25787b4d68054",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/19-proof-layer\")",
  "kind": "Result",
  "cites": [
    "bafyreib7fpigg6otslqtgnoajgwxpwc4t6ul75re37eoxodtws3emj55si"
  ],
  "rev": "223mssx6mnclo",
  "seq": 4,
  "of": 12,
  "text_len": 842,
  "content": "a764626f6479a166526573756c74a164746578746065636974657381d82a582500017112203f2bd06379d392e13335c049ad77d85c9fa8bff624dfc8ebb873b4b64627bd9266617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c727266632f31392d70726f6f662d6c617965726961727469666163747381a166436f6d6d697478286237396234303035383364383066656537313338396266633462663333313530373736376339626569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658c74929a1be"
}
---

Language probe (thermite-syntax parser, temporary test, deleted after): A+B1 is closer to wiring than to a language change. SHIPPED: prop fn/lemma/proof for/witness parse in an ORDINARY file with no attribute or flag and emit addresses (sum.proof.ensures#1:Forge, witness#1:Forge); a bodiless fn with a mandatory contract parses and yields Fn(sum, body=false) under #[boundary]; every downstream consumer already branches on body.is_none() (closure.rs, mutation.rs, verified_build.rs, address.rs); a post-parse desugar pass exists (thermite-syntax/src/desugar.rs) which is the merge mechanism 'body for f' needs. ABSENT: 'body for f' is a parse error at item dispatch; 'witness for f' is a parse error; and 'proof for no_such_function' PARSES CLEAN — nothing validates the target exists. Measured at the parse layer only; no verus/lean run.
---8<---
---
{
  "v": 2,
  "cid": "bafyreidce5dfuk64dm6copkyt474zeoaqb4y6bh5ruadisy5dapf5wlq6u",
  "sig": "1a26296170018d6e27b8a7a64dbe53bb3b5ac8d1e7aa68a0ce25e7e2b9d3a17f735af1ada7bce95319e028675387b22de0ae6311a0f67c6d17838b7e9b9cfb7c",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/19-proof-layer\")",
  "kind": "Observation",
  "cites": [
    "bafyreicugxwrdjwygrru7bc5srbb6ibkvtuz6vupsaq3rzziql2earp73y"
  ],
  "rev": "223mstaf4oxhy",
  "seq": 5,
  "of": 12,
  "text_len": 515,
  "content": "a764626f6479a16b4f62736572766174696f6ea164746578746065636974657381d82a582500017112205435ed11a6d834634f845d94421f202aace99f568f9021b8e72882f44045ffde66617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c727266632f31392d70726f6f662d6c617965726961727469666163747381a166436f6d6d697478283263393236303439393935326265326562343833316631303066613964653263313537363138333669776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658c9962a7529"
}
---

CORRECTION to the recorded claim that separating a specification from an implementation is blocked on there being no module system. The committed RFC section 9 splits that move into three forms and only the third is blocked: the body moving out binds by name, the contract moving out binds by name, and only refinement into a named abstract model needs mod/impl/trait, which surface-grammar.md REQ-1 lists as having no production. The cited claim treated all three as one blocked move and stands only for the third.
---8<---
---
{
  "v": 2,
  "cid": "bafyreiblsi5xhnmswiugr6alfpfcut5fscaddwx44x3y75wj3r6caagcse",
  "sig": "ca24fb177654e8f01e1b3c49cd21082440aa1b9cdfd2e74ff9e7c0be3bc4966c4fccd824c4920a33f1a249d307f6320ae3f2257f7ea9d09b0ca320fc92eea686",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/19-proof-layer\")",
  "kind": "Decision",
  "cites": [
    "bafyreib7fpigg6otslqtgnoajgwxpwc4t6ul75re37eoxodtws3emj55si"
  ],
  "rev": "223mstafqi6wn",
  "seq": 6,
  "of": 12,
  "text_len": 1152,
  "content": "a764626f6479a1684465636973696f6ea164746578746065636974657381d82a582500017112203f2bd06379d392e13335c049ad77d85c9fa8bff624dfc8ebb873b4b64627bd9266617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c727266632f31392d70726f6f662d6c617965726961727469666163747381a166436f6d6d697478283263393236303439393935326265326562343833316631303066613964653263313537363138333669776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658c997670d29"
}
---

Three constructs settled by the design pass, beyond the layering rule itself. (1) CLAUSE LABELS: item-scoped, optional, and required on any clause the proof layer references. An ordinal is a coordinate, so adding a loop renumbers f.loop#2.keeps#1 and adding an ensures moves what ensures#2 discharges; item scoping makes the address lose its ordinal segment rather than renaming it. The slot already exists because Clause.bv is an annotation that sits outside text, so semantic addresses remain unchanged. Optional rather than mandatory follows the RFC-9 argument: a mechanical pass over 547 sites would emit ensures_1, which is the ordinal with extra steps. (2) LABELS ARE CONTRACT, NOT ANNOTATION: they enter the projection of forge review, so a mislabelled clause misleads a reviewer in a way an ordinal cannot. Recorded because it runs AGAINST telos/surface-serves-agents rather than with it. (3) admits/excludes carry a visible subject: elided means the item, named means that clause. excludes is the per-conjunct form of vacuous_precondition, which is one boolean for a whole precondition today. Renaming inhabit to admits is 10 sites in 4 files.
---8<---
---
{
  "v": 2,
  "cid": "bafyreighsia6vzbw2ooofgv44zfeef3ufy6uixyfrptz663ro6bxzaukpa",
  "sig": "17c8580c1c92dd329c1bfc8a24e4ca39fc6acf3c374e68fac9952030228be34907ea0342bc39c6428a57a416f60d9bda8cb283ea217ad81764d7280f3b99546a",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/19-proof-layer\")",
  "kind": "Decision",
  "cites": [
    "bafyreib7fpigg6otslqtgnoajgwxpwc4t6ul75re37eoxodtws3emj55si"
  ],
  "rev": "223mstag45uqf",
  "seq": 7,
  "of": 12,
  "text_len": 1051,
  "content": "a764626f6479a1684465636973696f6ea164746578746065636974657381d82a582500017112203f2bd06379d392e13335c049ad77d85c9fa8bff624dfc8ebb873b4b64627bd9266617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c727266632f31392d70726f6f662d6c617965726961727469666163747381a166436f6d6d697478283263393236303439393935326265326562343833316631303066613964653263313537363138333669776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658c99821ea43"
}
---

The falsify budget leaves the source, reversing where the original spec put it. Thermite already files effort measurements outside the oracle: --rlimit is a flag, and the burn receipt is oracle-EXCLUDED on the stated ground that re-authoring a proof legitimately changes its committed-token count without changing what was proven. The falsify budget measures the same kind of thing and was handled the opposite way on both axes, and being written in source is what forced covenant_evidence into oracle_subset, where max_correct.cert.json pins falsify_generated 2002 as a result. The asymmetry that justified the source placement (lowering --rlimit is self-punishing, lowering falsify is self-rewarding) dissolves once the covenant is read as an economy gate rather than a rung: a refutation is decisive and never degrades, and finding nothing carries no assurance to weaken. Raised by the maintainer as a standing objection to the construct sitting in the language at all, on the ground that it is a statement about a random checker process component.
---8<---
---
{
  "v": 2,
  "cid": "bafyreicmlie2ny7xlqzp7kefaqpo7m2gd6pbo2bxxmrmwtjxmuxf3mbnj4",
  "sig": "50a3df9ee1d43cf9ee68c7422783113044e69472fc77bf311fcaf5f8d880fbf401141b630d00d24ffc9f4bfa69aacea2640b0938c6f50907a85412821d8c7d28",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/19-proof-layer\")",
  "kind": "Result",
  "cites": [
    "bafyreieuziqff2vx3ns7nehoqvesaexmkqz2qugi2o3ffpyno34koigncu"
  ],
  "rev": "223mstagqgiem",
  "seq": 8,
  "of": 12,
  "text_len": 823,
  "content": "a764626f6479a166526573756c74a164746578746065636974657381d82a5825000171122094ca2052eab7db65f690ee85492012ec5433a850c8d3b652bf0d76f8a720cd1566617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c727266632f31392d70726f6f662d6c617965726961727469666163747381a166436f6d6d697478283263393236303439393935326265326562343833316631303066613964653263313537363138333669776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658c999663579"
}
---

Design pass output, committed and open for review. Commit 40e1dea5 on branch rfc/19-proof-layer, one file .design/rfcs/0019-the-proof-layer.md, +842 lines, no code and no .th touched. Status draft/unfiled like RFC-8..14 and RFC-17..18, introduces [] so the registry is untouched. All nine gates/*.py exit 0 on the branch, including doc-drift (57 CURRENT, no re-pin needed because the RFC governs nothing) and rfc-check. Open as PR 35 into staging on the fork at maxinelevesque/Thermite3-staging, MERGEABLE, asking a direction check on the five questions of section 0 rather than a merge of any capability. Eight parser probes are recorded in the appendix, which states that no verus or lean run was made, so every certification claim in the document is cited from a REQ status table or a corpus header rather than produced.
---8<---
---
{
  "v": 2,
  "cid": "bafyreidglocobxr6vtny2fofhpp2ccpnn2kqnu7ou7znvogovlii75esli",
  "sig": "2e54bbe5472a37991f5d75abc4e2fe4c859b788f078c2fa1fd3b7c41c308bb021b7e847ccb5b8a422e79858e259a4a5786441f8d72e7f0a3a65b98d4cf03fe5b",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/19-proof-layer\")",
  "kind": "Status",
  "cites": [
    "bafyreicmlie2ny7xlqzp7kefaqpo7m2gd6pbo2bxxmrmwtjxmuxf3mbnj4"
  ],
  "rev": "223mstagqzd7g",
  "seq": 9,
  "of": 12,
  "content": "a764626f6479a166537461747573a16576616c7565644f70656e65636974657381d82a582500017112204c5a09a6e3f75c32ffa885041eefb3461f9e176837bb22cb4d37652e5db02d4f66617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c727266632f31392d70726f6f662d6c617965726961727469666163747381a166436f6d6d697478283263393236303439393935326265326562343833316631303066613964653263313537363138333669776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658c9996f8c69"
}
---
---8<---
---
{
  "v": 2,
  "cid": "bafyreic5vwbzmwq524ohv7g3q2wu3l7z3z5fxhrgvslvebnpx4wdx455ka",
  "sig": "a53ebeb8fc28cf0595e09994fd0eafa82020dd7fb7d247f64c10221ebfcdf3c24240a91c6aec99e673117c6000cbd5dce9368dd149cae21d6d7f188dd0e2d3c9",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/19-proof-layer\")",
  "kind": "Decision",
  "cites": [
    "bafyreicmlie2ny7xlqzp7kefaqpo7m2gd6pbo2bxxmrmwtjxmuxf3mbnj4"
  ],
  "rev": "223mstajncp4i",
  "seq": 10,
  "of": 12,
  "text_len": 743,
  "content": "a764626f6479a1684465636973696f6ea164746578746065636974657381d82a582500017112204c5a09a6e3f75c32ffa885041eefb3461f9e176837bb22cb4d37652e5db02d4f66617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c727266632f31392d70726f6f662d6c617965726961727469666163747381a166436f6d6d697478283263393236303439393935326265326562343833316631303066613964653263313537363138333669776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658c99f3453d6"
}
---

The atom chain for RFC-19 terminates at design-pass and does NOT continue to requirement-register, deliberately. day next reports requirement-register as the successor needing a design-doc, but RFC-19 is status draft/unfiled with introduces [], so there is no requirement to register and the registry stays untouched. This matches how RFC-8..14 and RFC-17..18 were handled and is the same trade those made: a design pass that asks a direction question is not a proposal, and registering requirements for an unfiled RFC would assert a commitment the document explicitly declines to make. Serves telos/residual-trust-is-named by keeping the asked-for thing (a direction check on the five questions of section 0) distinct from a filed obligation.
---8<---
---
{
  "v": 2,
  "cid": "bafyreicba2muhavfm64phseyizt42p2wyd33sx6gcmvxnkxhcthlxc7qtm",
  "sig": "0fe988f3249c8190c624f06273e6b5ba9f8916e17c54f6f0b19a1d7ead87414b646752a66ca2fdacc179c8c245ee6eccfb3d98f50c0c204d62502362322a1dc2",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/19-proof-layer\")",
  "kind": "Publication",
  "cites": [],
  "rev": "223mstc3crrwh",
  "seq": 11,
  "of": 12,
  "content": "a764626f6479a16b5075626c69636174696f6ea1656c6179657267476974547265656563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c727266632f31392d70726f6f662d6c617965726961727469666163747381a166436f6d6d697478283064326262383966313137376134646362336663373939653036353462373166613462343332373169776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658ca028bdf0d"
}
---
