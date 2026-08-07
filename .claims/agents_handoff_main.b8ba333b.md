---
{
  "v": 2,
  "cid": "bafyreicsk742ev6szfj7b4un3myaquld4relk442tqacagznxch6d2e3wm",
  "sig": "a11bc5ddd1d66d62fce4a616fabcc5a0487d6c9a1f7b150cacf716955e36c5bb471c18c186ad9cf7c73cc0e862da15430f17b7fa7d69deb25b8dceb879c38370",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"agents/handoff/main\")",
  "kind": "Retraction",
  "cites": [],
  "rev": "223msj7ojwtda",
  "seq": 0,
  "of": 5,
  "content": "a764626f6479a16a52657472616374696f6ea16a73757065727365646573d82a58250001711220d52481040abc0a3eb25f6356c958190ac53efafe8937567290b669ca9f32a6d06563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c736167656e74732f68616e646f66662f6d61696e6961727469666163747381a166436f6d6d697478283163323366333330373064333539656534353432346433666639393233303163626333666432353069776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b0006587968fe648f"
}
---
---8<---
---
{
  "v": 2,
  "cid": "bafyreif6ycatt5ucggtngcwvda6r3y54srevhnq6sl244he4epqktntesy",
  "sig": "b186e89451313ecbaf9b82fca3958d292744c45ca76e3afbd69cabd68fad75f003a37978e2ee3ce8eac9d0bfef3dfa1b700f36cee1546406982e79f7ed2d1e2a",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"agents/handoff/main\")",
  "kind": "Observation",
  "cites": [],
  "rev": "223msj7ok6wjn",
  "seq": 1,
  "of": 5,
  "text_len": 1048,
  "content": "a764626f6479a16b4f62736572766174696f6ea16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c736167656e74732f68616e646f66662f6d61696e6961727469666163747381a166436f6d6d697478283163323366333330373064333539656534353432346433666639393233303163626333666432353069776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658796902711e"
}
---

STATE at 2026-08-07. The anchor is implemented and evidenced; what remains is not implementation. anchor/implementation carries two commits - 39045f22 (front end) and 0dc65eb3 (corpus, tests, fixtures, evidence: 254 files, +2189/-2008) - both pushed. Certification matches the pin item for item including the failures, the AST comparison RFC-6 asks for passes over the whole corpus and all 596 rewritten Rust literals, and cargo test --workspace has an EMPTY set difference of failing test names against 84d276e7. Two PRs open in the fork and unmerged: #11 (in-tree kernel removal plus --target kernel renamed --target freestanding; zero regressions, build/doc-drift/req-registry all green) and #12 (this session's claims). main now has branch protection: PRs required, 0 approvals so solo merges work, no force-push, no deletion, admin bypass available. Issues #1-#4 mirror upstream #115-#118 about the layer #11 removes - #2/#3/#4 mooted, #1 substantially answered - and must NOT be closed in the fork, because mirrors track the upstream reports.
---8<---
---
{
  "v": 2,
  "cid": "bafyreiargaqy2dk73yqenzc5r5ulz5bds2ak4do2bw7kk3uvhndo2sxwde",
  "sig": "55307f32254aae56a54aa949b53a2e065785db144182332e789cf10f50a2925e38f420cccd7620e730d878a50a2fc611a6948c4193b74d857f611f61498eff30",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"agents/handoff/main\")",
  "kind": "Plan",
  "cites": [
    "bafyreif6ycatt5ucggtngcwvda6r3y54srevhnq6sl244he4epqktntesy"
  ],
  "rev": "223msj7p3sxpe",
  "seq": 2,
  "of": 5,
  "text_len": 1111,
  "content": "a764626f6479a164506c616ea164746578746065636974657381d82a58250001711220bec08139f68231a6d30ad5183d1de3bc944953b61e92f5ce1c9c23e0a9b6649666617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c736167656e74732f68616e646f66662f6d61696e6961727469666163747381a166436f6d6d697478283163323366333330373064333539656534353432346433666639393233303163626333666432353069776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658796a1c756d"
}
---

NEXT, in order. (1) RFC-15, harness-agnostic tooling: a clone should be bare - no .claude/, .crosslink/, .mcp.json, .claims/ active at root - with a justfile materializing a contributor's own choice (just stack claude, just stack day). 39 tracked files come out of upstream plus this fork's layer. Staged unfiled like RFC-8..14 until the anchor lands; propose the telos alongside it. Write it around TWO populations rather than one: config (.claude/, .crosslink/, .mcp.json) is inert files a stack copies into place, while process VOCABULARY - day's atoms, teloi, tensions - currently travels through .claims/, which conflates 'reasoning worth sharing' with 'vocabulary the tool needs to run'. day doctor reports 18 atoms and nine of them are witness subjects that exist only so the graph composes. A day pack install separates the two and .claims/ goes back to being only the former; the nine witness atoms are published TEMPORARILY for exactly that reason and their removal is a step of RFC-15, reasoning in practice. (2) Then the anchor sequence: atom/stage-gate -> atom/residual-trust -> atom/upstream-file.
---8<---
---
{
  "v": 2,
  "cid": "bafyreids3qkeajte4hcqoywwnlct52zupsx5mk2dzlw4ambgzkenc2dkie",
  "sig": "62ec731a437914f6cd606be1f8a3c7a8781c2d32fdc915b0e3ea002e353011da2082a8047affb1505f52272dc8de6aec0ad9a5ba4e06d917704dfdf11c5cfe4f",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"agents/handoff/main\")",
  "kind": "Observation",
  "cites": [
    "bafyreif6ycatt5ucggtngcwvda6r3y54srevhnq6sl244he4epqktntesy"
  ],
  "rev": "223msj7p43cje",
  "seq": 3,
  "of": 5,
  "text_len": 1106,
  "content": "a764626f6479a16b4f62736572766174696f6ea164746578746065636974657381d82a58250001711220bec08139f68231a6d30ad5183d1de3bc944953b61e92f5ce1c9c23e0a9b6649666617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c736167656e74732f68616e646f66662f6d61696e6961727469666163747381a166436f6d6d697478283163323366333330373064333539656534353432346433666639393233303163626333666432353069776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658796a20a129"
}
---

BEFORE FILING the anchor upstream, three things a reviewer must be TOLD rather than left to find in a 254-file diff. (a) Squash 39045f22 and 0dc65eb3 into one commit: RFC-6 says the migration and the parser change land as one PR, and this fork's practice is one commit per RFC so the revision derives from git. They are separate only because the implementation PR is not filed yet - PR #128 upstream is the RFC DOCUMENT on rfc/full-words, single commit d042c17c. (b) The scope is larger than RFC-6 states: 2910 clause sites, not 2074, from four measurement gaps each recorded on atom/conformance-evidence with its cause. The RFC's own numbers need correcting before it is re-filed, or the first reviewer to count finds the discrepancy. (c) 90 .md design docs still describe the v2 surface; outside RFC-6's stated scope, but doc-drift is a gate, so settle it before upstream-file. A fourth item evaporates if #11 lands first: thermite-kernel's frozen contract digest moved because it is sha256 over a Debug rendering carrying clause spans, and was re-pinned with a dated note - the crate goes away with #11.
---8<---
---
{
  "v": 2,
  "cid": "bafyreibceg5ueuvpd57x4c6q27zlikdviqcm5rxzclfe46isc67sr7xei4",
  "sig": "a0698e97438dcd7c0da62c7fd4a19acd17fab74138c337a05316575704b61c8e22ad0d6f283f5175ed178463e5e99581703c4738108fc8564a6478cc5b6346ce",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"agents/handoff/main\")",
  "kind": "Publication",
  "cites": [],
  "rev": "223msj7pmpstt",
  "seq": 4,
  "of": 5,
  "content": "a764626f6479a16b5075626c69636174696f6ea1656c6179657267476974547265656563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c736167656e74732f68616e646f66662f6d61696e6961727469666163747381a166436f6d6d697478283163323366333330373064333539656534353432346433666639393233303163626333666432353069776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658796b2ae279"
}
---
