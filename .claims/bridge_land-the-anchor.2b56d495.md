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
  "of": 2,
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
  "of": 2,
  "content": "a764626f6479a16b5075626c69636174696f6ea1656c6179657267476974547265656563697465738066617574686f72a26364696478396469643a6b65793a7a446e61656b7679686443524d315431797731425167736f7945766d6f79573237616e3361387279675a50716267653256656167656e74f6677375626a656374a1654c6f63616c766272696467652f6c616e642d7468652d616e63686f726961727469666163747381a166436f6d6d697478283139323965666439666232306261663862363130646138316561653262333962646238353238346669776f726b7370616365a169576f726b73706163657840323231666338316537643461376464326536633934303931323961393438623464633565326263393262626365383634663833623734353734316135346539366b7265636f726465645f61741b0006586d22ae3dbd"
}
---
