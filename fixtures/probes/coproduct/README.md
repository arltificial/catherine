# Probe 3 — coproduct exhaustiveness (pre-registration)

**Purpose.** Pre-register the G3 claim — "a rogue variant row and a
dual-covered row under a declared closed coproduct are accepted
silently, and only a clean closed table *should* be silent" — against
named fixtures, before any kernel coproduct code exists.

**Responsibilities.** Hold the committed fault set, the embedded
expectation catalog, the comparison rule, both executed baseline arms,
and the observed RED result. The control entry in
`ledger/probes/coproduct/` embeds these rows verbatim; the probe entry
against the same rows was recorded as seq 3, after the kernel check
landed. This directory is the substrate both are judged against.

**Rationale.** `add-coproduct-exhaustiveness` design Decision 5: the
capability claim is a probe first and a feature second, so a conventional
arm that already catches the faults would kill the change before merge.
Recording the current binary's silence *first* makes that comparison
arithmetic instead of rhetorical.

## Status

| Field | Value |
|---|---|
| Pre-registered | 2026-11-16 (RED task 1, `add-coproduct-exhaustiveness`) |
| Catalog | `fixtures/probes/coproduct/faults.json`, sha256 `388cefe6e8a894be114e7eccc87393e00850ecdde94078c027a9e05b4ebf0e4b` |
| Ledger entry | seq 1 (`0001-84847b2d.json`) the RED control; seq 2 (`0002-d717d7fa.json`) the re-recorded control pinning the registered catalog; seq 3 (`0003-2709a990.json`) the probe entry, `survived` |
| Probe entry | recorded as seq 3 against the same frozen rows: two unique detections, verdict `survived` |
| Kernel state | `schema.coproducts` is executable (format 1.5): coverage and disjointness failures fire `violation` findings |
| Frozen | an edit to a fixture or a catalog row is a superseding ledger entry, never a silent change |

The kernel-side `catalog_hash` pin is carried by seq 2 and seq 3: the
ledger validator derives it from `PROBE_IDS` / `probe_expectations` in
`crates/catherine-ledger/src/catalog.rs`, where `probe-coproduct` is
registered, and the registration reproduces the four catalog rows below
verbatim and in this order (control row first). The RED control (seq 1)
pre-dates the registration, so its content address was the file digest
`f18a5566…`; after the conventional baseline citation was corrected to
the artifact that actually resolves (see below) the file address is the
digest above.

## Fault set

| Fixture | Fault | Defect | Model hash | File sha256 |
|---|---|---|---|---|
| `payments-three-variants.ct.json` | `clean-closed-table` | `Method = card \| transfer \| cash`, every row covered exactly once | `sha256:e7a06c0b6a572e29c1a719ad4d5dffb0de056ca0516ce4764163a443303011ab` | `f68f291a41480aea3c5c6c5988368a865b80afa1d36859a8f22ccb03e399a791` |
| `payments-rogue-voucher.ct.json` | `rogue-variant` | fourth `Method` row `voucher` in no injection's image | `sha256:2e10d916c73f13383e706a8ff2396c76f24aabf827df2272bebefe939e225f47` | `97c13fd59668a7a65cbdb4ee3bea17486f7a52ce018f2b6358435710cd879553` |
| `payments-dual-covered.ct.json` | `dual-covered` | `Method` row `shared` in the image of both `card_inj` and `transfer_inj` | `sha256:1a55c46fd9e0a7c06dd95ad055f572d1a89cd31f7d0c2ab75e6054fea7fc5246` | `2238252e54ee2c24450d8778565e6d35c619e6cd86511e3af921f958cdb1b0c9` |
| `floor-keys-caught.ct.json` | `floor-keys-caught` (control) | `Card` rows `c1`, `c2` both collapse onto `Method/card` — `card_inj` not injective | `sha256:76dbedc228187a05fc6d606680f91f5b2cc23a509c844f9b41e2bf40e1e26fe2` | `566ba17799c006dbf1147e96d7e0ef3658b99a1775eea839a140031c7b1d5c01` |

The three payments documents are re-authored from the same corpus
transcripts the gap suite's `coproduct_payments.ct.json` encodes
(`kimi/1a1dee7a`, `claude/b417f19b`): the clean table, the rogue
`voucher` row and the dual-covered `shared` row are separated so each
arm is one document. The gap-suite fixture is **not** modified — it is
another ticket's case and is cited here only as the conventional
baseline.

Every document declares `format_version` `1.3`, the newest minor whose
known semantics this build carries: `schema.keys` is executable, and
`schema.coproducts` is an ignored field.

## Pre-registered catalog

Four rows, embedded verbatim in the control entry; the GREEN probe entry
embeds the same rows.

```text
fault          floor-keys-caught
series_class   behavioral
catherine_class finding
detail         floor control — two variant rows (`Card` c1, c2) collapse onto one coproduct row (`Method` card), a defect the incumbent `schema.keys` injectivity check names in full (violation, key 0, rows [0, 1]); the coproduct surface (coverage/disjointness of `Method`'s rows) cannot express injection injectivity, so the probe must show zero unique detections here (design Decision 5)

fault          rogue-variant
series_class   behavioral
catherine_class finding
detail         a rogue fourth variant — the `voucher` row of `Method` lies in the image of no declared injection of the accepted `Method` coproduct (`card_inj`, `transfer_inj`, `cash_inj`); the pre-registered expectation is one violation/executed naming the uncovered recipient row (gap G3: today the row is silently accepted, exit 0, findings [])

fault          dual-covered
series_class   behavioral
catherine_class finding
detail         a dual-covered row — `Method` row `shared` lies in the image of both `card_inj` and `transfer_inj`, falsifying the declared disjointness of an accepted coproduct; the pre-registered expectation is one violation/executed naming the row and both injections (today silently accepted, exit 0, findings [])

fault          clean-closed-table
series_class   behavioral
catherine_class finding
detail         a clean closed table — `Method` = card | transfer | cash with every row covered exactly once; the pre-registered expectation is zero findings (the negative side of both defect arms: the check must not fire on a correct declaration)

```

## Comparison rule (fixed before the run)

An *element* is one named fault witness. An arm identifies an element
only by naming it in its executed output:

- `coproduct:Method@<row>:uncovered` — a coproduct row in no
  injection's image;
- `coproduct:Method@<row>:dual(<inj_a>,<inj_b>)` — a coproduct row in
  two injections' images;
- `keys:<object>:<arrow>@rows<i,j>` — a `keys` injectivity collision.

`unique_detections = probe elements − baseline-identified elements`. An
arm that emits no finding identifies no element. The verdict follows the
recorded rule of `harness::derive_probe_verdict`: empty unique set →
`killed`; non-empty unique set with an executed baseline arm →
`survived`.

## Baseline arms (committed before the run, executed)

**Conventional arm** — the hand-authored gap-suite case
`coproduct-absence`, which pins today's behaviour as a recorded
limitation:

- `crates/catherine-cli/tests/expressibility_gaps/cases/analysis.rs`
  sha256 `8dc2d5258df113d915079c7d6d20271c7ba72e18d97c907564ab06193b752da4`
  (`coproduct-absence`, at the recorded RED revision `bda3819`)
- `crates/catherine-cli/tests/fixtures/gap_suite/coproduct_payments.ct.json`
  sha256 `95fc2ac80cd192b784c675e0889e08467fd16a37f3864ced433c9a484702dc62`
- `docs/ref/expressibility-dispositions.md` sha256
  `a97df1237d2399656acb43f29d539d68406a83d9c47baede4caec91d672fd341`

All three artifacts resolve at the recorded RED revision `bda3819`; the
pre-registration's file identity was corrected before any probe-entry
commit (the earlier citation named a `cases.rs` that never existed in
git, its `2630d4be…` digest matching no blob at any revision).

Executed first, exactly the command the case itself drives:

```
./target/debug/catherine check crates/catherine-cli/tests/fixtures/gap_suite/coproduct_payments.ct.json --json
# exit 0, findings [], warnings []
```

Detections: **none**. The arm asserts silence; silence is its detection
set. It does not name the `voucher` row or the `shared` row.

**Incumbent keys arm (floor control)** — the shipped `schema.keys`
injectivity check, pinned by the committed key-violation case:

- `crates/catherine-cli/tests/fixtures/key_violation.ct.json` sha256
  `10f9fbe10766322d6c60dbf165a82e21831e38bfbf7ae30e4caf47d9e666a9f6`
- `crates/catherine-core/tests/keys_checks.rs` sha256
  `6044d0ac4ed6e580d82635c76d207aee5ceed264c38c9e84b96bbcdc9d3f894c`
- binary `target/debug/catherine` sha256
  `819d798f808ef2935a3e44b718eb56461821ef43409073cec22bf346f4b4844c`
  (version 0.1.0, revision `79cef70`)

```
./target/debug/catherine check fixtures/probes/coproduct/floor-keys-caught.ct.json --json
# exit 3, findings [{"label":"executed","kind":"violation","witness":{"key":0,"rows":[0,1]}}]
```

Detections: `keys:Card:card_inj@rows0,1` — both collapsed variant rows
named. The control's premise: the incumbent check already catches this
variant defect, so the coproduct probe must add nothing here.

## Observed RED run (2026-11-16)

Binary `target/debug/catherine` sha256
`819d798f808ef2935a3e44b718eb56461821ef43409073cec22bf346f4b4844c`
(identical digest before and after the batch).

| Command | Exit | Findings |
|---|---|---|
| `check fixtures/probes/coproduct/payments-three-variants.ct.json --json` | 0 | `[]` |
| `check fixtures/probes/coproduct/payments-rogue-voucher.ct.json --json` | 0 | `[]` |
| `check fixtures/probes/coproduct/payments-dual-covered.ct.json --json` | 0 | `[]` |
| `check fixtures/probes/coproduct/floor-keys-caught.ct.json --json` | 3 | one `violation`/`executed`, `{"key":0,"rows":[0,1]}` |
| `check crates/catherine-cli/tests/fixtures/gap_suite/coproduct_payments.ct.json --json` | 0 | `[]` |
| `check crates/catherine-cli/tests/fixtures/key_violation.ct.json --json` | 3 | one `violation`/`executed`, `{"key":0,"rows":[0,1]}` |

The rogue `voucher` row and the dual-covered `shared` row are accepted
silently — the G3 premise, observed. The coproduct check itself names
neither, because it is not built yet: the two rows are recorded as
**pre-capability observations**, not as a probe verdict. The probe entry
that turns them into a comparison is appended by GREEN task 11.

The identical six commands were re-run after an unrelated concurrent
rebuild (binary sha256
`44f45bf05e850cf8a8f3458a8e73bd7fe7a9ba69b8ddc71963840f3f0a95ebb6`)
with byte-identical findings and warnings; the ledger entry pins
`819d798f…`, the digest verified before and after the recorded batch.

## Recorded state

- **seq 1, control — `killed`.** The floor fixture's single finding is
  the incumbent `keys` violation, already named by the executed keys
  arm: `unique_detections` = ∅. The control self-negates, as the
  validator requires, before any positive entry may be recorded.
- **seq 2, control — `killed`.** The control re-recorded once the
  registered catalog existed, pinning `catalog_hash
  sha256:a4b0451c…`; the floor element is again named only by the
  incumbent keys arm, so the unique set stays empty.
- **seq 3, probe — `survived`.** The built check names
  `coproduct:Method@3:uncovered` (rogue `voucher`) and
  `coproduct:Method@2:dual(card_inj,transfer_inj)` (doubly covered
  `shared`); the executed conventional arm identifies neither, so both
  are unique detections. The clean closed table stays silent, and the
  floor element is not counted (the keys baseline names it, which is why
  the control self-negates).

The seq-3 entry records the conventional-baseline citation correction:
the pre-registered arm named `crates/catherine-cli/tests/expressibility_gaps/cases.rs`
with sha256 `2630d4be…`, an artifact that never existed in git; the case
lives in `crates/catherine-cli/tests/expressibility_gaps/cases/analysis.rs`
at the RED revision `bda3819` (sha256 `8dc2d525…`). The verdict and the
unique-detection arithmetic are unchanged — the same executed arm
(gap-suite fixture, exit 0, findings `[]`) identifies none of the probe
elements.

## Falsifiers

- The incumbent `keys` check stops naming the collapse in
  `floor-keys-caught.ct.json` — the control's premise is gone and the
  control must be re-recorded as a superseding entry.
- A clean closed table fires any finding on this build — the
  `clean-closed-table` expectation is wrong.
- A rogue or dual-covered document fires any finding other than the
  pre-registered coproduct violation — record the discrepancy, never
  edit the catalog.
- At GREEN: the built check names neither `voucher` nor `shared` — the
  probe is `killed` and the capability does not merge.

## Execution

```
./target/debug/catherine check fixtures/probes/coproduct/payments-three-variants.ct.json --json
./target/debug/catherine check fixtures/probes/coproduct/payments-rogue-voucher.ct.json --json
./target/debug/catherine check fixtures/probes/coproduct/payments-dual-covered.ct.json --json
./target/debug/catherine check fixtures/probes/coproduct/floor-keys-caught.ct.json --json
./target/debug/catherine check crates/catherine-cli/tests/fixtures/gap_suite/coproduct_payments.ct.json --json
./target/debug/catherine check crates/catherine-cli/tests/fixtures/key_violation.ct.json --json
target/debug/ledger-check ledger/probes/coproduct
```