---
{
  "v": 2,
  "cid": "bafyreib42kniwukpb6rp3swrzavyxb5632ky3wkrqtq3hp2gzbd6brf4ay",
  "sig": "35bbc93029ea2a16649a834da641ad4984f75a68ed335fe3fa654d7894105a654adf38afe39c518e2b6788e573ace9f9137d822b08c86ed2922ede83667654b5",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/15-harness-agnostic-tooling\")",
  "kind": "Observation",
  "cites": [],
  "rev": "223msjaleudxa",
  "seq": 0,
  "of": 14,
  "text_len": 1369,
  "content": "a764626f6479a16b4f62736572766174696f6ea16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283438323632313131636236383137626365616333633237643165343065666663653034323962653569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065879a2ad26b8"
}
---

CI ITSELF requires the Claude Code control plane, which is a constraint RFC-15 has to answer rather than a detail. The checks job in .github/workflows/ci.yml runs a control-plane gate - tooling/control-plane-check.py - which reads .claude/settings.json and requires three wirings present: PostToolUse on Read to spec-discipline.py, and PreToolUse on Write and Edit to both spec-discipline.py and anti-pattern-gate.py. Probed rather than assumed: on a tree carrying tooling/ but no .claude/, it exits 1 with UNPARSEABLE, file not found, no hook is wired. So a BARE clone turns CI red at that gate, and moving .claude/ behind just stack claude is not sufficient on its own - the gate must become conditional on a stack being installed, or the harness-neutral clone is the one that fails CI, which inverts the whole argument for it. A second harness gate sits in the same job: the skill budget gate, cargo run -p thermite-skill --check-budget, a 6000-token budget over the generated THERMITE.skill.md. The two populations now have counts. Config - .claude/, .crosslink/, .mcp.json - is 39 tracked files, all of them upstream at 84d276e7, and it is CI-enforced. Vocabulary - .claims/ - is 36 files, ZERO upstream, this fork only, and enforced by nothing. That asymmetry is the argument: the population CI depends on is not the population RFC-15 was written to separate out.
---8<---
---
{
  "v": 2,
  "cid": "bafyreib6x6ql3aictxa3gdut5aagn6c2fhuuqxahkqeld53nfxc7o3zbba",
  "sig": "5df265e3cb4f32ad6d9008420740b6cc4c2c967b5b26b449a289d3a4c9e92a2b7ca0866d597c3e800dc596bf369e6d81aed0537186174c0227fbe00167e4e37f",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/15-harness-agnostic-tooling\")",
  "kind": "Subject",
  "cites": [],
  "rev": "223msjaleyoiz",
  "seq": 1,
  "of": 14,
  "content": "a764626f6479a1675375626a656374a2657469746c6578205246432d31353a206861726e6573732d61676e6f7374696320746f6f6c696e676c7375626a6563745f6b696e6464496465616563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283438323632313131636236383137626365616333633237643165343065666663653034323962653569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065879a2af50a4"
}
---
---8<---
---
{
  "v": 2,
  "cid": "bafyreiazimfcwpxhlwho5fsdbbkvtucsdsjvevp4atsyutiehwsuhtropi",
  "sig": "9cf50eb8aa1f74abe3ce257a9b8debf9c8c9be3c4ad42c758cc895860d717a8047d1a6d56f852f9486972826392d88d2adf3c28024c1b2bda1b23f5a4375612d",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/15-harness-agnostic-tooling\")",
  "kind": "Observation",
  "cites": [
    "bafyreib42kniwukpb6rp3swrzavyxb5632ky3wkrqtq3hp2gzbd6brf4ay"
  ],
  "rev": "223msjalxl72p",
  "seq": 2,
  "of": 14,
  "text_len": 690,
  "content": "a764626f6479a16b4f62736572766174696f6ea164746578746065636974657381d82a582500017112203cd29a8b514f0fa2fdcad1c82b8b87bede958dd95184e1b3bf46c847e0c4bc0666617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283438323632313131636236383137626365616333633237643165343065666663653034323962653569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065879a3d892fb"
}
---

ACToR specifically, since it is the natural suspect. The four agent definitions - .claude/agents/acto-builder.md, acto-critic.md, acto-fixer.md, acto-doc-author.md - are UPSTREAM tracked files, part of the 39, but nothing in CI invokes them. grep over .github/workflows for agent-facing references returns only comments plus the two gates already named. So ACToR is harness-specific config that a bare clone must be able to omit, and it is NOT a source of any CI failure. That splits the config population once more: config CI depends on (settings.json hook wiring, THERMITE.skill.md) versus config it merely carries (agents, crosslink rules, .mcp.json). Only the first blocks a bare clone.
---8<---
---
{
  "v": 2,
  "cid": "bafyreibpujcwdjcgkrzgdzbaklm4c3qwhzap5jvtwte7zs5uez2akz64qa",
  "sig": "16be7042f9cfb87a4b5b716cad5a56332cf6bd97e212629670222f1105fd9aec14eaac4a90e6e23a73407dfd52b4de901687245c654effd26115c553399cbf99",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/15-harness-agnostic-tooling\")",
  "kind": "Observation",
  "cites": [
    "bafyreib42kniwukpb6rp3swrzavyxb5632ky3wkrqtq3hp2gzbd6brf4ay"
  ],
  "rev": "223msjbcp4pxe",
  "seq": 3,
  "of": 14,
  "text_len": 1555,
  "content": "a764626f6479a16b4f62736572766174696f6ea164746578746065636974657381d82a582500017112203cd29a8b514f0fa2fdcad1c82b8b87bede958dd95184e1b3bf46c847e0c4bc0666617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283438323632313131636236383137626365616333633237643165343065666663653034323962653569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065879d1515728"
}
---

The same constraint from the other side, and it is now an ARGUMENT FOR RFC-15 rather than an obstacle to it. PR 12 touches only .claims/ - 16 files, zero under .claude/ or tooling/ - yet its checks job goes red, at a different step from PR 11: the doc-drift tripwire, DRIFT on .design/tooling/control-plane.md, whose governed patterns are .claude/agents/*.md, .claude/settings.json and tooling/control-plane-check.py. Fault established by running the gate at three points with python3.12, since the system python3 is 3.9 and doc-drift exits 3 INCONCLUSIVE on missing tomllib: upstream main 84d276e7 exits 0, this fork main 23931b9f exits 1, the PR branch inherits the 1. So the drift predates the PR and belongs to the fork process layer - 54d9bb92 wired day and kan into .claude/settings.json and e5ea0911 restored it by appending, +16 lines, two SessionStart entries invoking day hook, purely additive. control-plane-check.py exits 0 both sides, so the AUDITED control plane never changed; the content pin simply digests the whole file. The general shape: this fork cannot add its own process wiring without drifting an upstream design doc, so every branch cut from main is born red on a gate that has nothing to do with what the branch changed. Under RFC-15, .claude/ is not tracked at root and a stack materializes it, so the governed pattern matches nothing a contributor edits and this class of failure stops being possible. That is the strongest concrete case for the RFC so far, because it is a cost already being paid rather than a projected one.
---8<---
---
{
  "v": 2,
  "cid": "bafyreiepkehbganwyc4ay5fhby6dh7er3hu4ely2keylozqhonzk6iotmy",
  "sig": "65fc9862345fb467b8d5c59942fd64f034bca0ee87954eebbca92f0cbb4cae115b1ee11d0739d91b552f08921bd5eb13de3289de9b3575e3bf6c5dfa83abd4f6",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/15-harness-agnostic-tooling\")",
  "kind": "Result",
  "cites": [
    "bafyreibpujcwdjcgkrzgdzbaklm4c3qwhzap5jvtwte7zs5uez2akz64qa"
  ],
  "rev": "223msjbd24vm5",
  "seq": 4,
  "of": 14,
  "text_len": 1109,
  "content": "a764626f6479a166526573756c74a164746578746065636974657381d82a582500017112202fa24561a446547261e42052d9c16e163e40fea6b3b4c9fccbb426740567dc8066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283438323632313131636236383137626365616333633237643165343065666663653034323962653569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065879d2016dbb"
}
---

Re-pinned .design/tooling/control-plane.md rather than reverting the wiring, because the appended entries are correct and the audited control plane is unchanged. audited-content-sha256 moves cdce9510 -> 02b575b6, with a dated note in the frontmatter recording what moved it, that control-plane-check.py exits 0 both sides, and that RFC-15 is where the class of drift goes away - the same shape as the thermite-kernel digest re-pin on the anchor branch, a moved hash deserves a reader-visible reason rather than a silent bump. The content pin digests the governed files, not the document, so editing the prose does not move the hash again. After the edit, at HEAD with python3.12: doc-drift exits 0, control-plane-check exits 0, req-status exits 0. Two gates read red locally and BOTH are the recorded Python 3.9 tomllib trap rather than defects - tooling/reqs exits 3 INCONCLUSIVE, and the one failing unittest, test_reqs_facade_supports_check_render_query, fails only because it shells out to that script. Identical at the branch point 23931b9f, and CI installs 3.11+, so neither is ours and neither is real.
---8<---
---
{
  "v": 2,
  "cid": "bafyreicaxjlh2pie7sexsfpbjlzs6b64vfln2stqacvkwk52yzbtmmr6vq",
  "sig": "9b1bf0c9610508dc79be92ad3a02520033727dec90039e01aceb9ddfce4b037f2707da797de01e8d7172d6a1689ea8ab74f895ca246d3a486c23f96d48ea3575",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/15-harness-agnostic-tooling\")",
  "kind": "Publication",
  "cites": [],
  "rev": "223msjbe3yyjh",
  "seq": 5,
  "of": 14,
  "content": "a764626f6479a16b5075626c69636174696f6ea1656c6179657267476974547265656563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283438323632313131636236383137626365616333633237643165343065666663653034323962653569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065879d41f7964"
}
---
---8<---
---
{
  "v": 2,
  "cid": "bafyreieyzgpis4eumapkhughxvlvouaf7eajxbdyopy6gkcxf3el7quhym",
  "sig": "fa66ae40d1e5cc3b78a2cb398c6f67518b9a9035a3b41586168fc24ce2d608b061aaca33cfa9949525bc5c5c44de56ed4b37958aafaa13a49edf4b70e5cc5a59",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/15-harness-agnostic-tooling\")",
  "kind": "Result",
  "cites": [
    "bafyreiepkehbganwyc4ay5fhby6dh7er3hu4ely2keylozqhonzk6iotmy"
  ],
  "rev": "223msjbfyxndz",
  "seq": 6,
  "of": 14,
  "text_len": 862,
  "content": "a764626f6479a166526573756c74a164746578746065636974657381d82a582500017112208f510e1301b6c0b80c74a70e3c33fc91d9e9c22f1a5130b766077372af21d36666617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283538303732313536356235333632303164303363633130633637663238623361396161353539383769776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065879d7eeccb3"
}
---

Both fork PRs updated and pushed. PR 11 gets b10edfa8, the orphaned-comment fix plus reformat, as an ordinary non-destructive push over 936e07e9. PR 12 gets two commits: efaf73d7 re-pinning the control-plane doc, and 58072156 publishing the anchor squash, the CI attribution and this subject into .claims/. Note for whoever files PR 11 upstream: b10edfa8 should be squashed into 936e07e9 first, for the same reason the anchor was squashed - one logical change is one commit, and a reviewer does not need to watch us fix our own formatting. That was left as a separate commit deliberately, because a plain push starts CI immediately and this is the FIRST run on that branch to get past cargo fmt into doc-drift, control-plane, req-status, req-registry, clippy, doctests and skill-budget. Until it reports, those gates remain unmeasured on PR 11 rather than green.
---8<---
---
{
  "v": 2,
  "cid": "bafyreiepzdkv54urcseeldn56ncp7ai5uja6t3mb3upxlidadfa4atikjm",
  "sig": "28447bf0d404e96a45d07c21359b87d3ac71480457a577e35b644fb237b9b387293bd354c8599abf4e3b0166039c8b4f4993abb1ca72ac9fe57f45f88c6ed372",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/15-harness-agnostic-tooling\")",
  "kind": "Observation",
  "cites": [
    "bafyreibpujcwdjcgkrzgdzbaklm4c3qwhzap5jvtwte7zs5uez2akz64qa"
  ],
  "rev": "223msjbuu5mu5",
  "seq": 7,
  "of": 14,
  "text_len": 1032,
  "content": "a764626f6479a16b4f62736572766174696f6ea164746578746065636974657381d82a582500017112202fa24561a446547261e42052d9c16e163e40fea6b3b4c9fccbb426740567dc8066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283538303732313536356235333632303164303363633130633637663238623361396161353539383769776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065879f5a1ca91"
}
---

The fork process layer makes an UNRELATED upstream-bound PR red, which is the sharpest form of the cost yet. After the fmt and re-pin commits, PR 11 is green on doc-drift when the branch is tested directly but still red in CI. The difference is that actions/checkout builds the PR MERGE commit, and PR 11 has base=main on this fork. main carries the day/kan wiring in .claude/settings.json whose control-plane.md re-pin lives only on PR 12 branch, so merging 22fda741 into main reproduces it exactly: doc-drift exits 1 with DRIFT on .design/tooling/control-plane.md and nothing else. PR 11 removes the in-tree kernel and touches no harness file at all, yet it cannot go green until a claims PR merges. Sequencing that follows: merge 12 first, then re-run 11, which needs no further commit. The deeper point for the RFC is that the process layer is not merely carried on main, it is IMPOSED on every branch that bases against main - including the ones whose whole purpose is to be cherry-picked upstream, where none of it may travel.
---8<---
---
{
  "v": 2,
  "cid": "bafyreib764iicmzinvxr3xnpbenziwre6fm4v3k3zf5acqpc3ocihuztqa",
  "sig": "5064873aae88d5886d1f7a5d480857f41be63a78043a4f86b79d6b0f8fd168063d902ac369dd066b35b30679f3ec0b858195080a46553dac0b5e7bbf54fde80c",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/15-harness-agnostic-tooling\")",
  "kind": "Result",
  "cites": [
    "bafyreiepzdkv54urcseeldn56ncp7ai5uja6t3mb3upxlidadfa4atikjm"
  ],
  "rev": "223msjcpqv5ec",
  "seq": 8,
  "of": 14,
  "text_len": 1564,
  "content": "a764626f6479a166526573756c74a164746578746065636974657381d82a582500017112208fc8d55ef2911488458dbdf344ff811da241e9ed81dd1f75a0601941c04d0a4b66617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283538303732313536356235333632303164303363633130633637663238623361396161353539383769776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b0006587a2b6d8c48"
}
---

RFC-15 drafted and staged: .design/rfcs/0015-harness-agnostic-tooling.md, commit 53f118bc on rfcs/thermite-3-set, unfiled like RFC-8..14 until the anchor lands. Written around the three MEASURED costs from this session rather than a tidiness argument - bare clone red on arrival because control-plane-check.py exits 1 without .claude/, a second tool drifting an upstream design doc because the content pin digests the whole settings.json, and that drift imposed through the PR merge commit on branches carrying none of it. And around the two populations, with the numbers verified rather than repeated: 39 tracked config files from upstream at 84d276e7, and day doctor reporting 18 atoms of which exactly nine are witness subjects declared in[] out[]. Section 4 states residual trust including the hole this RELOCATES rather than closes - once settings.json is generated, nothing pins the generated file to its pack, the same class as crosslink #93. Section 7 proposes telos/the-clone-is-neutral: a contributor tooling is their choice, the repository neither assumes it nor requires it, a gate may verify a harness that is CLAIMED but never demand one be present, and the test is that a clone with no agent tooling installed is green. Its tension with it-lands-upstream is named rather than resolved, since rearranging a maintainer repository to serve a contributor who has not arrived yet is a harder sell than a language feature. Not adopted into the working telos set - that is a separate decision. rfc-check, doc-drift and req-status each exit 0 on the branch.
---8<---
---
{
  "v": 2,
  "cid": "bafyreibc3rnoiylz53izmqx2bkohlftmbffvn3cpqbfcawqsy3rtfpu6tu",
  "sig": "3e9cac3c0e92b3fd159aff750f7d3a972985a1b9d75a93deb037dffbdee00cb6694da507e4278fb97a3c404d37e37052f45ef62b10c4bdd511b83d7799f6e8a9",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/15-harness-agnostic-tooling\")",
  "kind": "Observation",
  "cites": [],
  "rev": "223msnfvccxsk",
  "seq": 9,
  "of": 14,
  "text_len": 1134,
  "content": "a764626f6479a16b4f62736572766174696f6ea16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283633366436353035333664613066333332343533636663653666346637653138666530353464643169776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b0006589af6847698"
}
---

A THIRD population in .claims/, which RFC-15 section 2 does not name. The RFC splits the tracked surface into config (inert files a stack copies into place) and process VOCABULARY (what day needs to run). The handoff is neither. agents/handoff/main is perishable OPERATIONAL state - branch SHAs, PR numbers, what the last session decided and why - addressed to the next session in this checkout rather than to anyone reading the project. It carried 175 lines and churned +116/-33 in one update. Publishing it put session working-notes into a shared, tracked file where every future session adds a large diff no reviewer wants. Resolved by unpublishing: handoffs are machine-local, and peer-to-peer coordination goes through issues, milestones and design docs, which are built for it. Worth folding into RFC-15 because it sharpens the thesis rather than complicating it - the question is not "reasoning versus vocabulary" but "what does this text OUTLIVE". Reasoning outlives the session and belongs shared; vocabulary outlives it and belongs in a pack; a handoff is consumed the moment it is read and verified, and belongs in neither.
---8<---
---
{
  "v": 2,
  "cid": "bafyreifmx4kdyqtn2dncppu35vutjgkefa2dmrooycd6vnoumzgobsc4ke",
  "sig": "ba5cb2e8cdc071f83ef7848a4a12a1d2ad3253441a4c06c6f634f3e560d6f15328b50fcc030a7f3c1f87743c458257c3655b53f3706dc3d338229cc0972b2467",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/15-harness-agnostic-tooling\")",
  "kind": "Publication",
  "cites": [],
  "rev": "223msnfvq2upm",
  "seq": 10,
  "of": 14,
  "content": "a764626f6479a16b5075626c69636174696f6ea1656c6179657267476974547265656563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283633366436353035333664613066333332343533636663653666346637653138666530353464643169776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b0006589af7606a3e"
}
---
---8<---
---
{
  "v": 2,
  "cid": "bafyreicfoqzrxulofviapyfqwwqn4xv4mmkunvm2xqmh6hduetv56kkm3q",
  "sig": "f7337fff5c2c0fe72b75feb24067cddbc2b9dcbb65f0d8219ca74940b64133f46f67a54334e058282706a4f20c69d3bca91b01ab8394815bf58a8dd137ff3f9f",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/15-harness-agnostic-tooling\")",
  "kind": "Decision",
  "cites": [],
  "rev": "223msr7iikits",
  "seq": 11,
  "of": 14,
  "text_len": 1167,
  "content": "a764626f6479a1684465636973696f6ea16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283530333237303839653436623965356535613063326639313166656434313264376635383332373269776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658b95ce83a3f"
}
---

SEQUENCE, kept as RFC-15 section 6 wrote it, against a proposed reordering. I proposed flipping steps 1 and 2 - make the gates stack-conditional FIRST, then git mv the config into opt-in/ - which removes the duplication window entirely and leaves no irreversible step at all, since a move is not a delete. The maintainer rejected it for a reason that outranks the tidiness: step 1 as written is landable UPSTREAM WITHOUT the gate changes, and the gate changes need buy-in from the other maintainers because they alter what a gate is permitted to demand. Reordering couples the uncontentious additive move to the contentious part and makes the whole thing wait on that conversation. So the duplication window between step 1 and step 3 is ACCEPTED, deliberately, as the price of decoupling. Two tracked copies of .claude/settings.json exist in that window and doc-drift pins only one of them, so an edit to one and not the other is invisible until step 3 - that is the same hole RFC-15 section 4 already names as 'Stack drift', now with a known duration rather than an unknown one. Serves telos/it-lands-upstream over telos/the-clone-is-neutral where they pulled apart.
---8<---
---
{
  "v": 2,
  "cid": "bafyreiblg7rkvk6j5ef27hu6pyzt7ctrs4wot36t7ak25v2ebm5i6kxbbq",
  "sig": "cbc56af4a65c7e7c08c608fd04f6f1f4d5e38c98ba1469d75a1421f843b23bc96c3cb5504a16449dda9734340b42b61cbcfc32fb6c73008c5e40dc729655d7c6",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/15-harness-agnostic-tooling\")",
  "kind": "Decision",
  "cites": [],
  "rev": "223msr7of7uhv",
  "seq": 12,
  "of": 14,
  "text_len": 1108,
  "content": "a764626f6479a1684465636973696f6ea16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283530333237303839653436623965356535613063326639313166656434313264376635383332373269776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658b968b2e94a"
}
---

THREE IMPLEMENTATION DECISIONS for step 1. (1) TASK RUNNER: justfile, as RFC-15 writes it, with the maintainer noting the intent to migrate everything from make to just eventually - so the two-runner state is a waypoint, not the destination, and filed as its own issue rather than left implicit. (2) UPSTREAM: none of opt-in/ travels yet. An upstream-bound branch carries the RFC-15 DOCUMENT only; the layout lands upstream as a second PR after a maintainer accepts the proposal. This is consistent with keeping the gate changes pending other maintainers' buy-in - the whole of RFC-15 is a proposal upstream until someone agrees, and shipping the layout ahead of that agreement would presume it. (3) MATERIALIZE SEMANTICS: 'just use <stack>' REFUSES when the target exists and differs from the stack, printing the diff, with --force to overwrite. Chosen over overwrite-always and over skip-if-present because it is the only one that SURFACES the drift the duplication window creates: overwrite silently discards a local edit, and skip-if-present silently serves a stale tree, which is a gate that fails open.
---8<---
---
{
  "v": 2,
  "cid": "bafyreieolfrsher26dml6vccxwfhtn6idydr73jxxnmb2oxn4mlw65plbu",
  "sig": "5ca031dbc0716a97e24bb478db6cd699eccfcf7a830189c52946152a02dfa7690f5319e4c3f060eccb5190383f1b5f78bb36d0e9b6b60f9cc8cfc7a09bceef2c",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"rfc/15-harness-agnostic-tooling\")",
  "kind": "Result",
  "cites": [],
  "rev": "223mstbq4vyoh",
  "seq": 13,
  "of": 14,
  "text_len": 2210,
  "content": "a764626f6479a166526573756c74a16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283665633930363364616362353536393031653538616665393639643634373863326334366362356469776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b000658c9ec2dfa10"
}
---

The six dangling hook entries are RFC-15's OWN EVIDENCE, and that is why they stay. Asked whether to delete them while pinning the interpreter, the maintainer said make it concordant with the RFC's intent; reading §1.4 settles it in the opposite direction from tidying. §1.4 is the RFC's strongest argument - it is the only one of the four not conditional on this fork having installed a second harness - and its exhibit is exactly that tracked .claude/settings.json wires nine hooks, six of which invoke scripts under the absent .claude/hooks/, so control-plane-check.py reports success over a control plane that is two-thirds dangling. Deleting them would remove the evidence for an unfiled RFC. Same test the nine witness atoms get: 'carries no information' and 'is load-bearing' are different questions, and they stay published until step 4 gives them a pack.

CORRECTED while there: §1.4 and §1.1 both said the three RESOLVING hooks point into tooling/. RFC-18 step 2's layout move put spec-discipline.py and anti-pattern-gate.py under gates/, and tooling/ now holds thermite3-migrate. The counts (nine wired, six dangling, three resolving) are unchanged and still true; only the directory name was stale. Fixed on chore/pin-the-interpreter, since the paragraph a maintainer evaluates the RFC on should not cite a directory that does not hold the files it names.

THE INTERPRETER PIN IS CONCORDANT, checked rather than assumed. telos/the-clone-is-neutral bars a gate from DEMANDING a harness; uv is the interpreter the repository's own gates run under, not a harness a contributor is asked to adopt, so demanding it is not the thing the telos forbids. And §3.2 makes opt-in/claude/ the copy 'just use claude' materializes from, so both settings.json copies were moved together and their hooks blocks verified byte-identical - a pack install cannot reintroduce the system-python invocation.

NOT DONE, and still RFC-15's own steps: §3.3's narrowing of doc-drift's control-plane route from the whole of .claude/settings.json to the stack that produces it; the stack-conditional branch in control-plane-check.py. Both wait on step 2, which is the maintainer-owned decision about what a gate may DEMAND.
