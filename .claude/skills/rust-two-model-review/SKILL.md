---
name: rust-two-model-review
description: >
  Review a Rust change with two independent agents — one Opus, one Sonnet —
  running the identical review task against this repo's Rust conventions, then
  converge their findings into one report. Use when the user asks for a "two-model
  review", "dual review", "two-agent review", or asks to review a Rust diff/branch/PR
  with both models. Higher-signal than a single reviewer: findings both models raise
  are high-confidence; a finding only one raises gets verified before it's acted on.
allowed-tools: Agent Bash(git:*) Bash(cargo:*) Read Grep Glob
---

# Rust two-model review

Run one Rust review task through two independent agents on different models, then
converge. The value is in the agreement structure: two models that independently
flag the same thing is strong signal; a lone finding is a lead to verify, not a
verdict. This skill reviews Rust changes against this workspace's conventions —
for non-Rust work it doesn't apply.

## 1. Scope the diff

Establish exactly what's under review before spawning anything, so both agents
see the same target.

- Default: the current branch vs `main` — `git diff main...HEAD` plus any
  uncommitted changes (`git status`, `git diff`).
- A specific PR: check it out (or `gh pr diff <n>`) and review that range.
- Capture the diff scope in a sentence you can paste into both prompts (e.g.
  "the diff on branch `add-amrap-rep-target` vs `main`").

Do not summarize the code for the agents — point them at the files and let each
read independently. Feeding them your summary collapses the two perspectives
into one.

## 2. Write ONE shared task

Both agents get the **identical** prompt — that's what makes the comparison
meaningful. The prompt must contain:

- The diff scope from step 1 and the repo path.
- Instruction to read the changed files themselves and the surrounding code.
- Instruction to review against this repo's Rust conventions: **consult the
  `rust-best-practices` skill** and the persistence/domain rules in `CLAUDE.md`.
  (Don't restate those rules here — the agents load both; just point them at it.)
- The output contract (step 3).

Fold in any extra focus the user named this run (e.g. "pay attention to the
migration path") into the shared prompt so both agents carry it.

## 3. Fix the output contract

Tell both agents to return findings as a ranked list, most-severe first, each
with:

- `file:line`
- one-sentence description of the defect
- a concrete failure scenario (inputs/state → wrong result), and
- a verdict: `CONFIRMED` (traced it to a real failure) or `PLAUSIBLE` (suspected,
  not proven).

Ask for an empty list when nothing survives scrutiny — a reviewer that always
finds something is noise.

## 4. Launch both agents in parallel

One message, two `Agent` calls with the **same `prompt`**, differing only in
`model`. Run them synchronously so you can converge as soon as both return.

```
Agent(subagent_type: "general-purpose", model: "opus",   run_in_background: false, prompt: <shared task>)
Agent(subagent_type: "general-purpose", model: "sonnet", run_in_background: false, prompt: <shared task>)
```

## 5. Converge

Merge the two finding lists into one report for the user:

- **Both models, same issue** → high confidence. Lead with these.
- **One model only** → verify it yourself (read the code, trace the scenario)
  before presenting. Mark clearly that a single model raised it and say whether
  your check confirms it.
- **Contradictions** → resolve by reading the code, not by vote.
- Dedupe overlapping findings; keep the sharper failure scenario.
- For each surviving finding, recommend: fix now, or document as a conscious
  decision (e.g. a pre-release breaking change with no migration).

Present the consolidated list. Do **not** auto-apply fixes unless the user asks —
this skill reviews; fixing is a separate, explicit step.
