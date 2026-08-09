---
{
  "v": 2,
  "cid": "bafyreidabrurmsswv33uymmt2bwkggic5nczx3dqvfrwi6htf3mup7qcma",
  "sig": "7da6c90967e123c3a24cbcbcee406140821e05ea147c242e4e707175b9d1ff4230996bc63fe4667f473b4a68e47a16489cc5a6e9a48987e57e4b8637c341b8a8",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/16-what-a-pin-should-pin\")",
  "kind": "Observation",
  "cites": [],
  "rev": "223msjynr2xkc",
  "seq": 0,
  "of": 9,
  "text_len": 1600,
  "content": "a764626f6479a16b4f62736572766174696f6ea16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781c7266632f31362d776861742d612d70696e2d73686f756c642d70696e6961727469666163747381a166436f6d6d697478283538303732313536356235333632303164303363633130633637663238623361396161353539383769776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b0006587fa7707590"
}
---

RFC-16 drafted and staged: .design/rfcs/0016-what-a-pin-should-pin.md, commit 1ad1d1c5 on rfcs/thermite-3-set, unfiled. doc-drift asks whether a document might now be false and answers whether the governed bytes moved; _digest is sha256 over read_bytes() of each governed file across whole-file globs, with no way to express a region. Argued from a measured rate rather than irritation: 41 drift findings across 4 root events in one day - 28 confirmed TRUE (documents describing req/ens/fx after RFC-6 deleted that surface), 7 undetermined without reading each, 6 confirmed FALSE. The 6 matter more than the ratio because they do not scatter: four from a formatting change whose export list is set-identical at 46 names, and two from sixteen additive lines wiring a second harness into .claude/settings.json where control-plane-check.py exits 0 on BOTH sides. That pair is the sharpest evidence available - same file, same commit, the executable check said TRUE and the hash said DRIFT. Both false shapes are properties of the PIN rather than the change, and neither overlaps the path that produced all 28 true findings, so they are removable without weakening what works. Proposal in three layers of decreasing confidence: pin a region or extraction, prefer a check to a pin where the claim is executable, transclude the surface rather than restate it. Generalizes RFC-15 section 3.3. Residual trust stated, including that narrowing genuinely trades recall for precision and that the 7 undetermined are NOT claimed false - if a meaningful share are true, the structural-confinement argument weakens.
---8<---
---
{
  "v": 2,
  "cid": "bafyreibzywgtukk4quqxik6njmd44i4t6umkb5gtayxql7ci3locsvwktu",
  "sig": "d1e274a9c65d119a528dc583be80a1bc286ee5bd3d82303f0217d7e59214bec7631b7cb675e9903952e0fd897db1929d1d05eb44890fac0afed03218e195dff9",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/16-what-a-pin-should-pin\")",
  "kind": "Subject",
  "cites": [],
  "rev": "223msjynr4xkv",
  "seq": 1,
  "of": 9,
  "content": "a764626f6479a1675375626a656374a2657469746c65781d5246432d31363a207768617420612070696e2073686f756c642070696e6c7375626a6563745f6b696e6464496465616563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781c7266632f31362d776861742d612d70696e2d73686f756c642d70696e6961727469666163747381a166436f6d6d697478283538303732313536356235333632303164303363633130633637663238623361396161353539383769776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b0006587fa77175ab"
}
---
---8<---
---
{
  "v": 2,
  "cid": "bafyreia6ko2hxfe2wqrek7s7uiyd7oj5r2bx3atwsxg5zxlrygv6n5b5ca",
  "sig": "2d34ccabd0223430c8c9af527082618e59c8793d8c4d4a2484765918acd63be90ec2ab1393b46506838bb329ae3d9e424c475265bf4a305c478f0b1c8853e563",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/16-what-a-pin-should-pin\")",
  "kind": "Result",
  "cites": [],
  "rev": "223mslkubsy6t",
  "seq": 2,
  "of": 9,
  "text_len": 1586,
  "content": "a764626f6479a166526573756c74a16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781c7266632f31362d776861742d612d70696e2d73686f756c642d70696e6961727469666163747381a166436f6d6d697478283538303732313536356235333632303164303363633130633637663238623361396161353539383769776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b0006588c347c77f6"
}
---

The Python-version trap is fixed at the source rather than worked around, as PR 14 on chore/uv-pinned-python, commit ce4795ac, cut from 84d276e7 so it stays cherry-pickable and touches no process-layer file. The key finding is that these gates have NO third-party dependencies - every tooling script is stdlib only - so this is interpreter SELECTION, not dependency resolution, and no lock file is added. What they have is a version requirement nothing in the repo stated: tomllib is 3.11+, needed by doc-drift.py, req-registry.py and spec-discipline.py. Three costs measured today rather than asserted. tooling/reqs check exits 3 INCONCLUSIVE instead of 1 on macOS system Python 3.9, which is why a genuinely failing req-registry gate was invisible locally and only surfaced in CI on PR 13. CI passes by accident of the runner image shipping 3.12, not because the repo asks for anything. And spec-discipline.py loads an EMPTY route table without tomllib, so every gated path looks unrouted and the hook blocks with no route table entry matches - failing closed, which is right, but naming the wrong cause and sending the reader to add a route that is already there. Fix: .python-version at 3.12, CI and make reaching it through uv run, and a require_toml_reader() that still fails closed but names the real cause. Verified through uv: doc-drift, control-plane-check, req-status, reqs check and the whole tooling suite exit 0, including test_reqs_facade_supports_check_render_query which had failed locally all day. Control retained: /usr/bin/python3 tooling/doc-drift.py still exits 3.
---8<---
---
{
  "v": 2,
  "cid": "bafyreicnerimrb6p72nt7jxgc7tzepq2gn5r32zgye2wnqnursw2rmlf2i",
  "sig": "68ca7f05e971816572b5245408230e3ec6fdb876ea8a9301acf22b131819e44b3f881664be681112d8b0e30ac8f844884e5b2921b30992d981a82aa17e7246ef",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/16-what-a-pin-should-pin\")",
  "kind": "Result",
  "cites": [
    "bafyreia6ko2hxfe2wqrek7s7uiyd7oj5r2bx3atwsxg5zxlrygv6n5b5ca"
  ],
  "rev": "223mslmskhftt",
  "seq": 3,
  "of": 9,
  "text_len": 690,
  "content": "a764626f6479a166526573756c74a164746578746065636974657381d82a582500017112201e53b47b949ab422457e5fa2303fb93d8e837d827695cddcdd71c1abe6f43d1066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781c7266632f31362d776861742d612d70696e2d73686f756c642d70696e6961727469666163747381a166436f6d6d697478283538303732313536356235333632303164303363633130633637663238623361396161353539383769776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b0006588cb106aed1"
}
---

PR 14 is 12 of 12 green after rebasing onto the post-kernel-removal main, at 546953d8. The uv step ran and resolved the pin in CI - the cache key reads setup-uv-1-x86_64-unknown-linux-gnu-3.12.3, so the interpreter is 3.12.3 because .python-version asks for it rather than because the runner image happens to ship it, which was the whole point. Every Python gate then passed through uv run, including req-registry, the gate that had been exiting 3 INCONCLUSIVE and therefore invisible on this machine all session. Note the check count fell from 13 to 12: kernel-image is gone, which is how the merged kernel removal shows up in a branch that rebased onto it. Ready to merge whenever wanted.
---8<---
---
{
  "v": 2,
  "cid": "bafyreidbjwlpgoig5ccgvsxnmkgiut5ujewsmtytzyc2aysokjdvaqklgi",
  "sig": "21c1041ac1689ad4392dfd1010909f7654bbb91573a11d813e09497d8c6a4ac8533cc336e810c76a55308dcb5450663ea23775f4c715ada596a9d7f6e58ca3c9",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/16-what-a-pin-should-pin\")",
  "kind": "Result",
  "cites": [],
  "rev": "223msmapsknvz",
  "seq": 4,
  "of": 9,
  "text_len": 1352,
  "content": "a764626f6479a166526573756c74a16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781c7266632f31362d776861742d612d70696e2d73686f756c642d70696e6961727469666163747381a166436f6d6d697478283538303732313536356235333632303164303363633130633637663238623361396161353539383769776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065891ab884f0e"
}
---

The seven undetermined drift findings are resolved: ALL FALSE. Tally is now 28 true and 13 false out of 41, none outstanding. RFC-16 revision 2 is e4d32dcf. The test that settled it was narrower than reading prose - for each doc, ask what the anchor actually CHANGED in the files it governs. Every changed line in verified_build.rs, composition.rs, cli.rs, closure.rs and thermite-syntax/src/lib.rs is the RFC-6 rename, including the ones that do NOT look like it because they are clause names built as data: format!("{}.ens#{}") became .ensures#, format!("{}.req") became .requires, and one module doc comment moved loop#1.inv#2 to loop#1.keeps#2. No behaviour moved, and none of the seven documents quotes an address segment, so none was falsified. This STRENGTHENS the proposal rather than weakening it: section 3 gains a third shape - a whole-file pin over source touched by a semantics-preserving global rename - and the three are one root cause wearing three faces, the pin digests more than the document depends on. Naming the principle is a stronger claim than enumerating symptoms. Section 6 drops the undetermined caveat and replaces it with a real one: 41 findings on one fork over one day is enough to show the shapes are real and not enough to size them, and all 28 true findings came from a single surface rename rather than routine work.
---8<---
---
{
  "v": 2,
  "cid": "bafyreidfn72ioey75foydkdgkwx6m3ztxnucdtjem5jxalmnwwu3np4lli",
  "sig": "12041664b3b75c8566b344dbf0685c42c7ddacf3ce56dc314f992a6745286dea344a18905b1eda75d18c0b7e8f4824de9c835e44e4c83aff5b7d7b88b00c1242",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/16-what-a-pin-should-pin\")",
  "kind": "Result",
  "cites": [],
  "rev": "223msmchs5f7m",
  "seq": 5,
  "of": 9,
  "text_len": 1521,
  "content": "a764626f6479a166526573756c74a16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781c7266632f31362d776861742d612d70696e2d73686f756c642d70696e6961727469666163747381a166436f6d6d697478283538303732313536356235333632303164303363633130633637663238623361396161353539383769776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658921b81ac39"
}
---

RFC-16 layer 1 is BUILT, as PR 16 on feat/doc-drift-narrow-pins, commit 869a530c. Two opt-in narrowing mechanisms in tooling/doc-drift.py: anchors, doc:begin(<doc-relpath>) to doc:end in a governed file narrowing that doc digest to the enclosed regions; and extractions, pin-extract: <path>=<extractor> in a doc header narrowing one file, with one extractor shipping - claude-hooks, digesting the hook entries the repository OWNS, meaning those whose command references a path under tooling/, which is exactly the set control-plane-check.py verifies. The no-flag-day property was PROVED rather than asserted: both algorithms run over one identical tree, 59 documents compared, 0 digest differences. Only a narrowed file contributes a mode marker to the digest, so an un-narrowed file contribution is byte-identical to v1 and every existing pin keeps its value. control-plane.md is the first route narrowed because it is where the cost was measured, and the elimination was verified live: appending a foreign hook now leaves doc-drift at 0 and control-plane-check at 0, while test N-5 pins the other half, that changing a repo-owned hook still drifts. Both failure modes are INCONCLUSIVE rather than clean - unclosed doc:begin exits 3, unknown extractor exits 3 naming it - because silently narrowing a pin is the exact failure the mechanism exists to prevent. Six oracle tests in the existing fixture convention, asserting which edits must and must not drift rather than any hash the tool prints. Layers 2 and 3 deferred.
---8<---
---
{
  "v": 2,
  "cid": "bafyreia6dav5ftie7ptckgp65flzqn54mz4yfurcqk4iwkgmbpg2eblmfi",
  "sig": "869ee7c7db01863501a0c521b70e29c2492937dedc1f839759ebd48ae0f2f9742ed67e84b84ff61c4b5d2cb7dfc1512fe55fc5cee7369325faabbd3ec44477f8",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/16-what-a-pin-should-pin\")",
  "kind": "Result",
  "cites": [],
  "rev": "223msmkdfwj7z",
  "seq": 6,
  "of": 9,
  "text_len": 1591,
  "content": "a764626f6479a166526573756c74a16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781c7266632f31362d776861742d612d70696e2d73686f756c642d70696e6961727469666163747381a166436f6d6d697478283538303732313536356235333632303164303363633130633637663238623361396161353539383769776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b0006589412be3c52"
}
---

RFC-17 is IMPLEMENTED as PR 17 on feat/rfc17-one-vocabulary, commit 5b059da4, based on rfc/full-words because the RFC is unreviewable before RFC-6 exists. TokKind::{Req,Ens,Fx,Inv,Dec} became {Requires,Ensures,Effects,Keeps,Measures}, Contract{req,ens,fx} became {requires,ensures,effects}, and LemmaItem{req,ens} became {requires,ensures}. 104 files, +449/-404. The completeness proof is the compiler and that is the whole point of separating this from RFC-6: the rename is type-directed, so cargo check --workspace --all-targets exiting 0 is not corroboration but the argument itself, since an unrenamed site does not compile. RFC-6 had no such luxury - string literals, a JSON population and clause-names-as-data are invisible to rustc. The compiler also arbitrated the FALSE positives, which is the other half of the value: a textual rename of .req/.ens/.fx hits types that legitimately own those names - ItemFact.fx, ExecClause.req, and four *ObligationFrame.req - and all 22 such sites surfaced as E0609 and were reverted, so those types keep their names. Scope grew by two fields during implementation: LemmaItem was not in the RFC table and appeared as a compile error; leaving it would have reinstated the exact split the RFC closes, so the TABLE was updated in 7cd0fea8 rather than the scope quietly widened. Verified: check, clippy -D warnings and fmt --check each 0; tests/golden/ changed 0 files and .th changed 0 files, which is section 3 predictions holding since Verus lowering never sees a Rust field name and the corpus is surface rather than AST. 45 design-doc pins moved.
---8<---
---
{
  "v": 2,
  "cid": "bafyreicehqajg5z5yh7y3eleimjkfpyukclgowhesyvhton4g3m4ak2cai",
  "sig": "727e9957c56253d92f303678f45165ba3c8a39516e3d4100034450e160a1b4395a7fe7df51200b29c321928171449ffdc1884c7e84f8f3e2a48b8b28dc15ae36",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/16-what-a-pin-should-pin\")",
  "kind": "Publication",
  "cites": [],
  "rev": "223msmrsm5yaa",
  "seq": 7,
  "of": 9,
  "content": "a764626f6479a16b5075626c69636174696f6ea1656c6179657267476974547265656563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781c7266632f31362d776861742d612d70696e2d73686f756c642d70696e6961727469666163747381a166436f6d6d697478283538303732313536356235333632303164303363633130633637663238623361396161353539383769776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065895f121f826"
}
---
---8<---
---
{
  "v": 2,
  "cid": "bafyreidngdb232zmp2elchy6hiicje37nwrnsfuvwtzmqjgck76fj6iueu",
  "sig": "ac708e19e04c16419b19747d3be76927e7c4eb620a7c40f30a0223112b448603601391f01f0ee5db0234a5ca97899e1d7b32127c0ad2b9b8cc823d7bcc9ea748",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/16-what-a-pin-should-pin\")",
  "kind": "Publication",
  "cites": [],
  "rev": "223msmrti3erb",
  "seq": 8,
  "of": 9,
  "content": "a764626f6479a16b5075626c69636174696f6ea1656c6179657267476974547265656563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781c7266632f31362d776861742d612d70696e2d73686f756c642d70696e6961727469666163747381a166436f6d6d697478286162333731663539323961633362333665306262663537636638346534306233643964366463386569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065895f2e0aa57"
}
---
