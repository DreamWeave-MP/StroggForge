# DOC-101: the five stages, and what a receipt proves

Status: **proposed RFC** for [DOC-101](https://github.com/DreamWeave-MP/StroggForge/issues/25).
It is blocked by SEC-101 (#21) and is drafted against that RFC's proposed boundaries. It goes to the
approver only after SEC-101 is ratified, and is revised if SEC-101 changes in ratification. It
decides nothing until the approver ratifies the list at the end. DOC-102 (#26) owns the receipt's
byte format; this RFC owns what a receipt must say and what it means.

## What the five stages are for

Every change goes Inspect → Implement → Strict → Style → Profile. Review cuts across all five. A
stage either produces evidence someone else can check, or says plainly that it didn't run and why.

The stages produce **evidence**, never **authorization**. A green receipt doesn't merge, release,
tag or sign anything (SEC-101, "Five things that are not the same thing"). Authorization comes only
from:

- the approver, or
- a standing authority the approver ratified, within its limits.

That holds however good the evidence looks.

## Stages

Each stage records one of these statuses, never a blank:

| Status | Means |
| --- | --- |
| `passed` | its exit criteria hold, with evidence |
| `failed` | its exit criteria don't hold; the evidence says how |
| `skipped` | allowed for this change's tier, with a reason |
| `not-applicable` | the stage has nothing to act on, with a reason (e.g. Profile for a typo fix) |
| `blocked` | can't run yet; names what it waits for |

`skipped` and `not-applicable` are not `passed`, and nothing may report them as passed.

| Stage | Entry | Exit (passed) | Evidence |
| --- | --- | --- | --- |
| **Inspect** | a change is proposed | the baseline is reproduced at an exact source SHA; a falsifiable hypothesis is written ("the forge agrees with the Rust on every corpus case"); the threat boundary is named, citing SEC-101's crossings where one is touched | SHAs of every repository read, the commands that reproduced the baseline and their results, the hypothesis |
| **Implement** | Inspect passed | the change is atomic (one concern, revertible on its own) and the tree builds | commits, and what was deliberately left out |
| **Strict** | Implement passed | the tests, types and negative cases the tier requires pass on the final SHA, and they exercise the hypothesis rather than nearby behavior | each command, its exit status and summary; mutation or negative checks where the tier requires them |
| **Style** | Strict passed | the project's own gates pass: `@jess/style`, fmt, Clippy pedantic, actionlint, docs conventions; generated artifacts are deterministic | commands and results |
| **Profile** | Style passed | before and after measured where the change can affect cost, or a reasoned no-impact statement where it can't | the machine, the build (a verified build only: no debug or no-LTO numbers), runs, spread |

**Review** is cross-cutting. It may happen during any stage, and it must be finished before the
change is presented for merge.

### When a stage fails

- A failure loops back to the earliest stage it invalidates: usually Implement, sometimes Inspect if
  the hypothesis was wrong.
- The failed attempt stays in the record. A receipt is append-only: it shows the failure, what
  changed, and the later pass.
- A flaky pass after a failure records both runs, and is not `passed` until the cause is named.
- A retry without a change is a re-run, recorded as one. Three failed re-runs of the same thing stop
  the stage as `failed`, not `blocked`.
- Profile is never "failed" merely because a number went the wrong way. It fails when the
  regression exceeds what the change's hypothesis allowed, or when it can't be measured as the
  tier requires.

## Risk tiers and what each demands

The tier is the highest that any part of the change touches.

| Tier | Touches | Strict | Review | Profile |
| --- | --- | --- | --- | --- |
| **R0** | prose or comments only, no behavior | docs build, links | self-review | `not-applicable` |
| **R1** | code in one package or crate, no privilege, no public contract | the package's tests and types, plus a test for the new behavior | self-review, with the diff re-read cold | no-impact statement, unless on a hot path |
| **R2** | a public contract, a runtime, a shared library, a parser, or user-visible behavior | R1, plus negative cases, plus a mutation or differential check that the tests aren't vacuous | one independent adversarial review | measured when the change sits on a path the hypothesis or users care about; else no-impact |
| **R3** | anything in SEC-101's crossing tables: a workflow, a release path, a credential, a trust root, a signing identity, a cross-repository contract | R2, plus the SEC-101 invariants the change touches, as tests | two independent reviews, or one plus the approver's own reading | as R2, plus the CI cost of the change |

- **Independent** means the reviewer didn't write the change, is given the code and the issue
  rather than the author's conclusions, and reports defects with reproductions.
- A finding ends in one of four dispositions, recorded in the receipt:
  - fixed;
  - upstreamed (with the link);
  - recorded as a divergence or residual (with the reason);
  - rejected (with the evidence).

## What a receipt must say

Format and storage are DOC-102's. Meaning is this RFC's. Every stage's record carries:

- the exact source SHA it ran on, for each repository involved;
- the tools and their versions, and the digest of any binary not built from those sources (e.g. the
  jess that ran the suite);
- each command exactly as run, with its exit status and the result observed;
- the stage status, and for `skipped`, `not-applicable` and `blocked`, the reason;
- what was **not run**, as its own list. An empty list is a claim too;
- findings and their dispositions, and residual risks;
- larger evidence by reference and digest, not copied (e.g. a `stroggforge.diagnostics/1` document
  or a differential receipt): `path@SHA` plus SHA-256.

A receipt never contains a secret, a token, the paper phrase or anything derived from it, or an
environment dump.

A receipt is evidence written by whoever ran the stage. It is untrusted input to anyone who reads it
later (SEC-101 T-PROMPT): a resuming agent re-checks the source SHAs and digests rather than
believing the prose.

### Resumable

Each stage's record stands alone. Given the receipt and the repository, a second agent can say:

- which stages passed, failed, were skipped or are blocked;
- which SHA each ran on;
- whether the tree has moved since.

If it has moved, the stages after the change are stale and must re-run. A receipt never says
"tests pass" without the SHA they passed on.

## Worked examples

**A typo fixed in a README (R0).**

| Stage | Record |
| --- | --- |
| Inspect | the SHA, and the line |
| Implement | one commit |
| Strict | the docs build, or `not-applicable` where the project has none |
| Style | the docs conventions |
| Profile | `not-applicable`: prose |

Review is self-review. No benchmark is run or owed.

**A jess runtime patch: the `@jess/yaml` layout channel, jess#30 (R2).**

- **Inspect:** the forge's nine `{}`/`[]` divergences reproduced at a StroggForge SHA. The
  hypothesis: a layout side channel makes them decidable without changing `decode`.
- **Implement:** separate commits for the feature and for each class of fix.
- **Strict:**
  - the package's suite and types;
  - a 4000-document generator, used as the mutation and differential check;
  - every YAML file in the local repositories decoded before and after.
- **Style:** `@jess/style` through the integration suite.
- **Profile:** `decode` measured 6–9% slower on a 3000-item document. That was recorded, and the
  hypothesis allowed it.
- **Review:** one independent review. Its findings were fixed in a further commit, and the rest were
  listed in the PR as left alone with reasons.

**A cross-repository release change: the tag-name injection fix SEC-101 proposes (R3).**

- **Inspect:** the splice cited at `corprus-crucible/action.yml:233,245,262`, and a tag name shown
  to pass `git check-ref-format`.
- **Implement:** one commit passing `release_name` through `env`.
- **Strict:**
  - SR-11 finds none of those lines;
  - a fixture tag containing `$(…)` runs as data in a scratch repository;
  - actionlint.
- **Style:** actionlint and AGENTS.md conventions.
- **Profile:** no-impact.
- **Review:** two independent reviews, then the approver's explicit go-ahead, because it changes a
  production workflow. The receipt is evidence for that go-ahead; it is not the go-ahead.

## The receipts written before this RFC

FOR-201 and FOR-202 carry receipts in FOR-102's provisional format, which predates this RFC. Once
this RFC is ratified, they are reconciled against it. Their nonconformances are fixed in those
branches, and each correction is listed in the PR. Neither merges until that's done. Known gaps to
check:

- a per-stage status with `skipped`/`not-applicable` distinguished;
- tool and binary digests for every run;
- an explicit "not run" list per stage;
- the tier each change was held to.

## Profile of this RFC

The cost is the time to write a receipt: minutes for R0 and R1, part of the work itself for R2 and
R3. That evidence is already being produced; this RFC fixes its shape. No tool is required by it.
DOC-102 may add a schema checker, and DOC-201 a receipt runner.

## Decisions for ratification

1. **The four tiers** (R0–R3) and what each demands, in the table above.
2. **The statuses,** including that `skipped` and `not-applicable` are never `passed`, and that a
   receipt is append-only.
3. **The receipt contents.** This includes the three proposals EP-FOR made on #25, now in the list
   above:
   - evidence by digest, not copy;
   - a mandatory "not run" list;
   - a mutation or differential check counting as Strict evidence at R2.
4. **Merge authority.** DOC-101's issue says "Human PR merge remains required". The approver has
   since granted a standing authority: the agent may merge its own PRs when CI is green and an
   independent review has passed. Releases, tags, signing, secrets and permissions, production
   workflows and consumer cutover always need the approver. Proposed wording: *a merge needs the
   approver, or a standing authority the approver ratified, within its limits; R3 changes always
   need the approver.* The approver chooses this wording, the issue's original, or an edit.
5. **Reconciliation.** FOR-201 and FOR-202 are held to this RFC before either merges, with no
   exception.
