# Guide tripwire corpus

The 34 `.rs` files are vendored verbatim from the Verus guide's example
sources — `verus-lang/verus` at tag `release/0.2026.11.22.1ce6a59`,
path `examples/guide/` (source commit `1ce6a59`). Verus is MIT-licensed
(see the upstream `LICENSE`); these files are upstream teaching
examples, not kernel code, and carry no catherine modifications.

`expectations.txt` records each file's outcome under the pinned release
(measured 2026-11-25, `scripts/check-verus-lane.sh` classification):
28 verify; 5 are library-style snippets the verifier driver rejects
with `E0601` (`main` function not found — they are meant to be read as
fragments, not programs); `higher_order_fns.rs` fails verification by
design (the guide's own call-to-non-static-function teaching example,
15 verified / 1 error).

Sync command (re-vendor at a new pin, then re-measure expectations
with the same pinned release):

```sh
git clone --depth 1 --filter=blob:none --sparse \
  https://github.com/verus-lang/verus /tmp/verus-src
cd /tmp/verus-src && git sparse-checkout set examples/guide
cp examples/guide/*.rs crates/catherine-verus/fixtures/guide/
# then re-run scripts/check-verus-lane.sh and reconcile expectations.txt
```

The lane fails on any outcome drift between `expectations.txt` and a
fresh run under the pinned release — that is the tripwire: an upstream
or toolchain regression surfaces here before it can silently change
what the lane's proof results mean.
