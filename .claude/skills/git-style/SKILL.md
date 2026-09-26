---
name: git-style
description: >
  How to shape commits, commit messages, branches and pull requests in this repo.
  Use before committing, writing a commit message, opening or describing a PR,
  deciding how to split work into PRs, rebasing a branch, or merging stacked PRs.
allowed-tools: Bash(git:*) Bash(gh:*) Read Grep
---

# Git style

A commit or PR is read by a reviewer with limited time. Every rule here protects
that time.

## Commits

- One commit, one intent. A message that needs "and" to join two intents is a
  sign of two changes — but only a sign; see the next section.
- Subject: short and phrased as a command — "Add set validation", not "Added…".
- Body: one or two sentences saying what the diff can't — the constraint that
  forced the shape, or the measurement behind a trade-off. Not a recap of the
  diff, a list of files, or numbers the reader can count.
- No `Co-Authored-By` or other attribution trailer.
- Each commit builds and passes the tests on its own. After rewriting a branch,
  check every commit, not only the last.

## A reviewer reads each line once

A reviewer should never read a line that a later commit, or a stacked PR,
replaces. The first reading is wasted.

- If a later commit would reword a comment an earlier one added, put the final
  wording in the earlier commit.
- If two intents change the same lines, splitting them can't avoid the double
  read — in either order, one version is shown and then replaced. Land them as one
  commit in one PR: name the main intent, and fold the other into a clause.
- Otherwise, order commits so each later one adds or removes, and never rewrites
  what an earlier one wrote.

## Pull requests

- Branch off `main` with a kebab-case name that describes the change.
- Keep PRs small, ideally under 500 lines of diff. Split larger work into a
  sequence of PRs, unless the pieces change the same lines.
- Description: the intent, and the one constraint that shaped the result. Not a
  tour of the diff, and not a list of what you ran. End it with the
  `🤖 Generated with [Claude Code](https://claude.com/claude-code)` line.

## Updating a branch

- Rewrite a branch freely until it merges, pushing with `--force-with-lease`.
  Never rewrite commits after they merge.
- When `main` has moved, rebase before asking for a review.
- After a rebase, look past the conflicts git reports. Hooks, CI files and docs
  written for something your branch replaces can merge cleanly and still be wrong.

## Stacked PRs

A stacked PR is based on another PR's branch instead of `main`.

- Merging the lower PR does not move the upper one to `main`. Merge from the top
  down, or retarget each upper PR as the one below it lands:
  `gh pr edit <n> --base main`.
- Never merge with `--delete-branch` while another open PR is based on that
  branch. GitHub closes the dependent PR.
