---
name: plan-commits
description: >
  Plan the sequence of commits a PR will consist of before implementing it, so
  the commits tell a reviewer the story of what is being built. Use before
  implementing a feature or any change that will take more than one commit, or
  when asked to plan commits. Skip it for a change that fits in one commit.
---

# Plan commits

A reviewer reads a PR one commit at a time. Planned up front, the commits read as
a story: each adds one step to what is being built, and none makes the reviewer
reread something an earlier one showed them.

## The rules a plan follows

- **One intent per commit.** Each commit moves the story forward by one step,
  and the reviewer should never have to ask what this commit is for.
- **Commits are additive.** A later commit may add to what an earlier one
  introduced: fill in a stub's body, add a field, a variant or a parameter. It
  never renames, removes, or changes the behaviour of what an earlier commit
  introduced. Needing to means the earlier commit was cut wrong.
- **Every commit is green.** Each builds and passes the checks on its own.
- **Tests go with the code** they cover, in the same commit.

One way a story can run — an illustration, not a template; let the change decide
its own steps:

1. Introduce the shape — the types, signatures or contract — with no behaviour
   yet.
2. Fill in the behaviour behind it.
3. Use it from the code that needs it.

The reviewer sees what is being built before how it works, and how it works
before where it is used. Each step adds to the one before and revises none.

## Steps

1. **Plan.** List the commits in order. Each entry is a subject phrased as a
   command ("Add set validation", not "Added…") and a draft body, rarely more
   than a line, saying what the diff can't.
2. **Approve.** Present the plan in chat and wait for the user's approval. Then
   create one task per approved commit.
3. **Implement** the tasks in order, one commit each, with the approved subject
   and body.
4. **Re-plan on drift.** If a commit turns out to need a change to something an
   earlier one introduced, stop. Explain what broke the plan, and propose a
   revised one for approval before going on. If that reshapes commits already
   made, re-check each one.
