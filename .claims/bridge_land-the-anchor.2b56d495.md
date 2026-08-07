---
{
  "v": 2,
  "cid": "bafyreico3hv5hvygv3ymfbpofwo2j72zduafz6wdictzqznhuvvptjfhba",
  "sig": "cb4423ef1d39f7290e794f62d17556a20b0a8b6a95907332876701bde6692e1c4426ba3aca33f5926c58897b76395125c8bb7e62e2158a5faca632b982a98efa",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"bridge/land-the-anchor\")",
  "kind": "Observation",
  "cites": [],
  "rev": "223mshojultam",
  "seq": 0,
  "of": 7,
  "text_len": 934,
  "content": "a764626f6479a16b4f62736572766174696f6ea16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c766272696467652f6c616e642d7468652d616e63686f726961727469666163747381a166436f6d6d697478283139323965666439666232306261663862363130646138316561653262333962646238353238346669776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b0006586d1fa8e3dc"
}
---

RFC-6 is filed and its front end is committed on anchor/implementation: 63 insertions and 62 deletions across five files, both crates building. What remains is evidence, not design. Migrate the corpus and certify against baseline item for item including the failures; migrate the 1527 clause sites inside Rust literals, which is three quarters of the work and whose write path does not exist yet; regenerate goldens; then run the test suite, which has never been run post-migration and is the one genuine unknown. Three productions are in RFC-6 and not in the spike - conjunct blocks, requires nothing, and one-or-more requires - and the corpus does not use them, so they do not block this bridge.

```day-bridge
{"telos":"it-lands-upstream","have":["design-doc","registered-requirements","implementation"],"plan":{"seq":[{"atom":"conformance-evidence"},{"atom":"stage-gate"},{"atom":"residual-trust"},{"atom":"upstream-file"}]}}
```

---8<---
---
{
  "v": 2,
  "cid": "bafyreig5qmhx64zlaheotfkwcmhsuxpbr7lh2pqidrlrvszhp2g4epkhqi",
  "sig": "2a2e806059d0ed911bec428a91bd1d9c8370e6728fc7174d8ce23e038160ceb27337fa002f5a2ad7658aca74bfa86e426cb0249be750761adcdf932d99823b7a",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"bridge/land-the-anchor\")",
  "kind": "Publication",
  "cites": [],
  "rev": "223msholewjml",
  "seq": 1,
  "of": 7,
  "content": "a764626f6479a16b5075626c69636174696f6ea1656c6179657267476974547265656563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c766272696467652f6c616e642d7468652d616e63686f726961727469666163747381a166436f6d6d697478283139323965666439666232306261663862363130646138316561653262333962646238353238346669776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b0006586d22ae3dbd"
}
---
---8<---
---
{
  "v": 2,
  "cid": "bafyreiaxqb5ya2cxpvceailh2aolxezmfadikshnk4ckgphwgydltgx2ba",
  "sig": "c8a3a36a13f739083918f0dbd648b1241c4ae61676520692b3fae9d7d1cc3a393cbf9836c959ccd5324e6fa73a6885bcf969c40998df4719136088a196793b7f",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"bridge/land-the-anchor\")",
  "kind": "Result",
  "cites": [],
  "rev": "223msjak6bc5i",
  "seq": 2,
  "of": 7,
  "text_len": 1001,
  "content": "a764626f6479a166526573756c74a16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c766272696467652f6c616e642d7468652d616e63686f726961727469666163747381a166436f6d6d697478283438323632313131636236383137626365616333633237643165343065666663653034323962653569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065879a0439fd4"
}
---

Squashed the anchor implementation into ONE commit, per RFC-6 landing as one PR and this fork practice of one commit per RFC so the revision derives from git. 39045f22 (front end) plus 0dc65eb3 (corpus, tests, fixtures, evidence) became 62750676, parent d042c17c - the RFC-6 DOCUMENT commit on upstream/rfc/full-words - so the branch is still cut from an upstream ref rather than from this fork main. Verified content-identical BEFORE moving the ref: tree 7bbdb405 on both sides, and git diff 62750676 0dc65eb3 is empty. The combined message keeps both original texts and leads with why they are one commit - neither half certifies alone, since no front end accepts the new surface until the parser lands and a migrated corpus cannot be checked against a parser that still spells req/ens/fx. Old SHAs are recorded HERE because the handoff observation cites them by name; after the force-push they are unreferenced on the remote. Reversal while 0dc65eb3 exists locally is pushing it back to the branch.
---8<---
---
{
  "v": 2,
  "cid": "bafyreig2uj2sqnpseshejvodwpykh6qy5wzwftxdncrr23awivz4nyjf5m",
  "sig": "2463f6c2da72ade0ec2e4fd7b90def5eacf5ec21042e65d7ecfceb34173d89cf060d000c8ba626abce8402e1157b15376622affa20b3ad3dc426cdb4b2a7d01e",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"bridge/land-the-anchor\")",
  "kind": "Observation",
  "cites": [
    "bafyreif6ycatt5ucggtngcwvda6r3y54srevhnq6sl244he4epqktntesy"
  ],
  "rev": "223msjakukyep",
  "seq": 3,
  "of": 7,
  "text_len": 1077,
  "content": "a764626f6479a16b4f62736572766174696f6ea164746578746065636974657381d82a58250001711220bec08139f68231a6d30ad5183d1de3bc944953b61e92f5ce1c9c23e0a9b6649666617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c766272696467652f6c616e642d7468652d616e63686f726961727469666163747381a166436f6d6d697478283438323632313131636236383137626365616333633237643165343065666663653034323962653569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065879a1a87813"
}
---

CI attribution for the two fork PRs, which each fail exactly ONE check. checks fails 19-22s in at its FIRST step, cargo fmt --all --check - not at any harness, control-plane or ACToR gate. Fault established before fixing: the base of chore/move-kernel-out is 84d276e7 (upstream main, PR 114), and all three files in the fmt diff - forge/tests/freestanding_target.rs, thermite-lower/tests/l3_library.rs, thermite-syntax/src/lib.rs - are touched by our single commit 936e07e9. The drift is mechanical fallout of the rename itself: --target freestanding and L3LibraryTarget::Freestanding push lines past the width, so rustfmt rewraps. Ours, not upstream and not the harness; upstream PRs 127/128/129 are green on checks, corroborating a clean base. Consequence for the record: the claim that PR 11 has build, doc-drift and req-registry all green is a LOCAL result, not a CI one. fmt is step one of that job and fails fast, so doc-drift, control-plane, req-status, req-registry, clippy, doctests and skill-budget never ran on 11 at all - its CI is SILENT on them rather than green.
---8<---
---
{
  "v": 2,
  "cid": "bafyreiatb6hlrzar62m4id7azsjw6qojtdjphrphpns2ljfxvjljskpjjm",
  "sig": "e6c05fa479d4e728a2a83db3ad66afcd11ce6fea30fec770d8eecebc10918f4e534c1005a5eebccaddc2029891e6d47067f5afa754d26a4052ad93ae114d41fc",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"bridge/land-the-anchor\")",
  "kind": "Result",
  "cites": [
    "bafyreiaxqb5ya2cxpvceailh2aolxezmfadikshnk4ckgphwgydltgx2ba"
  ],
  "rev": "223msjardrcba",
  "seq": 4,
  "of": 7,
  "text_len": 603,
  "content": "a764626f6479a166526573756c74a164746578746065636974657381d82a5825000171122017807b8068577d44402167d01cbb932c28068548ed5704a33cf63606b99afa0866617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c766272696467652f6c616e642d7468652d616e63686f726961727469666163747381a166436f6d6d697478283438323632313131636236383137626365616333633237643165343065666663653034323962653569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065879ae9ba037"
}
---

Force-push done and verified. origin/anchor/implementation is 62750676; the GitHub commit list for the branch now reads 62750676, d042c17c, 112b2dee, so the two-commit history is gone from the remote and the branch is one commit on top of the RFC-6 document commit. The lease held - the remote was still 0dc65eb3 at push time, so the guard was never exercised. Nothing else referenced the old SHAs: no PR in either repo had the branch as head or base, no tag, no other branch contained 39045f22 or 0dc65eb3, and the branch is unprotected. The t3-work worktree stayed clean because the tree did not move.
---8<---
---
{
  "v": 2,
  "cid": "bafyreicvvl67cngj4vep3vl3qjpq6l52df5amjwi54fylhmqlsk7v63z3u",
  "sig": "04a3b544f9dae1a3ff5f7d5ebd3f07ba8b244bdd83834fbdceba3df51b60f23038c1478d12f042d47aecaf3b914a5e3cbf5ad4088b8a2166c6cd57dbca44d921",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"bridge/land-the-anchor\")",
  "kind": "Observation",
  "cites": [
    "bafyreig2uj2sqnpseshejvodwpykh6qy5wzwftxdncrr23awivz4nyjf5m"
  ],
  "rev": "223msjavueioc",
  "seq": 5,
  "of": 7,
  "text_len": 1145,
  "content": "a764626f6479a16b4f62736572766174696f6ea164746578746065636974657381d82a58250001711220daa2752835f2248e44d5c3b3f0a3fa18edb362cee368a31d6c164573c6e125eb66617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c766272696467652f6c616e642d7468652d616e63686f726961727469666163747381a166436f6d6d697478283438323632313131636236383137626365616333633237643165343065666663653034323962653569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065879b7a5396e"
}
---

The fmt gate on PR 11 was not only cosmetic - it surfaced a real defect in 936e07e9. forge/src/vacuity.rs removed the match arm Effect::Platform(_) => {} but left its two-line comment - Kernel platform authorities are narrow, domain-scoped grants and do not participate in the hosted maximal-row heuristic - dangling before the closing brace. rustfmt cannot leave a trailing comment there, so it reflows it onto the PRECEDING arm, where it then falsely describes Effect::Term. Accepting cargo fmt output blindly would therefore have committed a wrong comment. The correct fix is deleting the orphan, since the arm it documents no longer exists. Fixed that way; the other three files are pure rewrapping caused by the rename lengthening lines - freestanding_target.rs, l3_library.rs, thermite-syntax/src/lib.rs. After the fix, cargo fmt --all --check exits 0 and cargo check on forge, thermite-lower and thermite-syntax with --all-targets exits 0, both verified directly rather than through a pipe. Generalizable: a formatter is a comment-attachment oracle for arm deletions - removing a match arm silently orphans the comment above the NEXT one.
---8<---
---
{
  "v": 2,
  "cid": "bafyreiedgk5ao6zpkac35mruwquuslpwgzy5iy5qrxuck73hngl2hcghyy",
  "sig": "d126036953338e76b4483b9075089ea86a09d622921540caf2438e7a56d447c2079bfec8fde0a640101d65c74a45d07f61cd64fbcd8720c16273c34e986e226c",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"bridge/land-the-anchor\")",
  "kind": "Publication",
  "cites": [],
  "rev": "223msjbe3s2c6",
  "seq": 6,
  "of": 7,
  "content": "a764626f6479a16b5075626c69636174696f6ea1656c6179657267476974547265656563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c766272696467652f6c616e642d7468652d616e63686f726961727469666163747381a166436f6d6d697478283438323632313131636236383137626365616333633237643165343065666663653034323962653569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065879d41c0084"
}
---
