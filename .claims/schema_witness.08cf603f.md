---
{
  "v": 2,
  "cid": "bafyreif62ca2qzcnucookcyei5qtfpm4gezgje3r45vthicuajy3y4zmza",
  "sig": "da81c6be69b5af83353748a9866cdee21ccba7756cc4bb4a8161aaab50f9c0124e2f237f6a2339ae57e69c5179c56f9b603b7b225803e08193eb88356ed91763",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"schema/witness\")",
  "kind": "Observation",
  "cites": [],
  "rev": "223mshoizwp7r",
  "seq": 0,
  "of": 2,
  "text_len": 1094,
  "content": "a764626f6479a16b4f62736572766174696f6ea16474657874606563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c6e736368656d612f7769746e6573736961727469666163747381a166436f6d6d697478283139323965666439666232306261663862363130646138316561653262333962646238353238346669776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b0006586d1dfe5426"
}
---

What evidences each atom's output type in this repo. Every entry is a probe day can run, so process position is inferred from artifacts rather than tracked.

```day-witness
{
  "probe-verdict": {"path": ".design/probes/*.th"},
  "spike-result": {"path": ".design/*-spikes.md"},
  "design-doc": {"path": ".design/rfcs/*.md"},
  "registered-requirements": {"path": ".design/reqs/registry.toml"},
  "implementation": {"command": "grep -q 'requires' thermite-syntax/src/lexer.rs"},
  "certification-result": {"path": "conformance/*.cert.json"},
  "gate-result": {"command": "uv run --python 3.11 tooling/rfc-check.py"},
  "trust-statement": {"command": "grep -rqi 'residual trust' .design/rfcs/"},
  "upstream-pr": {"command": "git log --oneline upstream/main..HEAD"}
}
```

The implementation probe checks the lexer for the renamed keyword rather than for a diff, because a diff against upstream is true on any branch that touches the front end and this needs to mean the rename specifically. A material witness is not an advanced telos: check what the witness CONTAINS, not that the probe passed.
---8<---
---
{
  "v": 2,
  "cid": "bafyreidaaftu4kimdbj7tgx5avufh35mtbqgjx5yl2cf7lhmi5npxpbvf4",
  "sig": "617310618b0b4554cae62c445c4868b4b42852a6df274aa60bae1ce98c5a5192228c1ffdd5c7a97a8fd68f8ad1313630b2cf14050fbc7ed657ca5ab2866e563e",
  "author": "did:key:zDnaekvyhdCRM1T1yw1BQgsoyEvmoyW27an3a8rygZPqbge2V",
  "subject": "Local(\"schema/witness\")",
  "kind": "Publication",
  "cites": [],
  "rev": "223msholer2fw",
  "seq": 1,
  "of": 2,
  "content": "a764626f6479a16b5075626c69636174696f6ea1656c6179657267476974547265656563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c6e736368656d612f7769746e6573736961727469666163747381a166436f6d6d697478283139323965666439666232306261663862363130646138316561653262333962646238353238346669776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b0006586d22ab80e4"
}
---
