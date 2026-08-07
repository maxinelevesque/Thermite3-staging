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
  "of": 6,
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
  "of": 6,
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
  "of": 6,
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
  "of": 6,
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
  "of": 6,
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
  "of": 6,
  "content": "a764626f6479a16b5075626c69636174696f6ea1656c6179657267476974547265656563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c781f7266632f31352d6861726e6573732d61676e6f737469632d746f6f6c696e676961727469666163747381a166436f6d6d697478283438323632313131636236383137626365616333633237643165343065666663653034323962653569776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b00065879d41f7964"
}
---
