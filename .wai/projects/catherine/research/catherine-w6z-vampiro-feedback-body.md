# vampiro 0.5.0 law-axis false negative: same-typed cross-origin join passes `check` (catherine B2)

## Summary

`vampiro check --full -p crates` on catherine @ 268c142 reported **0 findings**, including `crates/catherine-cli/src/runners.rs`, which contained a real seam-composition bug of exactly the class vampiro targets: a cross-origin join of two same-typed values — `TypedInstance::build(&model.schema, &second.instance)` — that is shape-valid (a `(&Schema, &Instance)` pair, so it compiles and "composes") but violates the provenance law **"schema and instance come from the same document."**

That bug (tracked as catherine-18t, fixed in catherine commit 8f3fbcc) conflated two independent documents' artifacts at a seam. vampiro 0.5.0's check, pointed at the full workspace, saw nothing.

## Why this matters

The failure is not shape-shaped; it is provenance-shaped. Type-level checking cannot see it because both arguments are correctly typed individually — the violation only exists in the *relationship* between arguments (their document origin). This is a concrete, reproduced false negative on the law axis: gate mode at `--severity-threshold high` would not have blocked the buggy push.

## Ask

Integrate law/prove into the CLI (`vampiro prove` is currently a placeholder) so a declared provenance law — e.g. *"arg0 schema and arg1 instance share one document origin"* — is evaluated at call sites and catches same-typed cross-origin joins like the one above. Once law checking lands and this class is caught, re-evaluate vampiro as a pre-commit gate (it is currently only wired as a complementary pre-push gate in catherine).

## Reproduction context

- Repo: catherine @ 268c142
- Command: `vampiro check --full -p crates`
- Result: 0 findings
- Bug location (pre-fix): `crates/catherine-cli/src/runners.rs` — `TypedInstance::build(&model.schema, &second.instance)` (catherine-18t, fixed in 8f3fbcc)

---

## Filing record (catherine-w6z)

### Dry-run (`vampiro feedback --kind bug --dry-run`, body piped via stdin) — EXIT=0

```
## Description

# vampiro 0.5.0 law-axis false negative: same-typed cross-origin join passes `check` (catherine B2)



## Environment

- tool: vampiro 0.5.0
- os/arch: linux/x86_64
- gh_version: gh version 2.102.0 (2026-09-30)
https://github.com/cli/cli/releases/tag/v2.102.0
- git_remote: https://github.com/arltificial/catherine.git
- git_branch: main
- git_dirty: true
- repo_state: wai beads openspec espectacular
- repro_hash: `d24c81a1ac0ef540`


Would file: gh issue create --repo charly/vampiro --title "[bug] feedback report" --label agent-reported, bug, has-repro
vampiro: dry-run — feedback would be filed to GitHub
```

### Real filing attempt (single attempt, per brief) — FAILED, EXIT=2

```
vampiro feedback: gh error: GraphQL: Could not resolve to a Repository with the name 'charly/vampiro'. (repository)
```

The upstream repo `charly/vampiro` is not resolvable via gh (repo does not exist under that name or is private/inaccessible to the authenticated account). No retry loop. Manual fallback link printed by the tool points at `https://github.com/charly/vampiro/issues/new?...` with the full pre-rendered body. This file is the surviving deliverable.
