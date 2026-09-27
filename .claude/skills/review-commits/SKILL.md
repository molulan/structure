---
name: review-commits
description: >
  Review a branch's commits before it is pushed, then hand them to the user
  for approval. Use when a branch's commits are done and it is about to be
  pushed or have a PR opened.
---

# Review commits

A branch leaves the machine only after its commits pass review and the user
approves them, so the story a PR reviewer reads is the one that was planned.

## What the review checks

- The rules in the `plan-commits` skill
  (`.claude/skills/plan-commits/SKILL.md`).
- The commit-message rules in CLAUDE.md's Git section.
- The commits match the approved plan, or the approved re-plan.
- `scripts/check-commits origin/main` passes: every commit is green.

## Steps

1. **Review.** Hand the review to a fresh subagent, one that did not write the
   commits. Give it the approved plan, and have it read the rules from their
   sources rather than a summary, go through
   `git log -p --reverse origin/main..` one commit at a time, and run
   `scripts/check-commits origin/main`. It reports each finding with the commit
   and the rule it breaks.
2. **Fix.** Rewrite the commits to fix the findings, folding each fix into the
   commit it belongs to, then review again with a new subagent. A finding that
   needs the story re-cut rather than a commit fixed is drift: re-plan with the
   user, as `plan-commits` describes. If a review after the second round of
   fixes still has findings, stop and bring them to the user.
3. **Hand over.** Once a review passes with no findings, report the commits
   and wait for the user's approval. Push the branch
   (`--force-with-lease` if it was pushed before) and open the PR only when
   the user says so.
