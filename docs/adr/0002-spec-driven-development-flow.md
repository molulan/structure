# 2. Define the spec-driven development flow

Date: 2026-11-23

## Status

Accepted

## Context

ADR 1 adopted spec-driven development but left the spec format and
workflow open. The goal behind it is a review process that keeps up
with AI-generated code: I want to spend my attention on what a feature
should do, not on proof-reading how it was done.

Two things shape the approach. First, a spec only works as the source
of truth if people actually read it, and agents implement from it, so
a bloated spec fails twice: nobody reads it, and both implementation
and verification get worse. Second, agents are not deterministic, so
an agent saying "this is correct" can never be fully trusted. Whatever
can be checked mechanically should be, and agents should only judge
what is left.

## Decision

1. **The spec is the source of truth, and it is committed before the
   code.** A spec is written for user-visible behaviour when code is
   written or rewritten; existing code is not specified after the
   fact.
2. **A spec stays lean.** It has a feature heading with a short
   purpose, then one section per functionality: the rule in a sentence
   or two, with a brief why where the rule needs one, and an optional
   `Critical:` marker naming one or more of `security`, `privacy`,
   `data` and `performance`. Each functionality holds its scenarios.
3. **Scenarios are titled GIVEN/WHEN/THEN examples that name their
   verifier.** The verifier is `types`, `test` or `review`, and shows
   how far a scenario can be trusted, strongest first. A functionality
   holds only when all its scenarios hold.
4. **The spec is closed-world.** Code must not do anything its spec
   does not define.
5. **Tests point at the spec, never the reverse.** A test carries its
   scenario's title as its name, in the idiomatic style of its
   language; matching ignores casing and separators.
6. **Deterministic checks first, agents for the rest.** Scripts check
   the spec's format and the links between scenarios and tests. Agents
   judge only what a script cannot.
7. **Specs are critiqued before they are committed.** Separate agents
   check clarity, consistency with other specs, leanness, and whether
   verifiers could be stronger or a functionality should be marked
   critical. They ask and suggest, and change the spec only once I
   have approved.
8. **Code is verified against the spec before pushing.** Fresh agents,
   which never see the implementer's reasoning, find the
   functionalities a change could affect, check each one's scenarios,
   and look for behaviour the spec does not define. The report lists
   problems, questions for me, code I should read myself (critical
   code and changed shared code), and what is met. A problem is
   resolved by fixing the code or changing the spec, never by
   accepting the drift. After any fix, the whole check runs again.
9. **The flow:**

       1. Define the spec
       2. Critique the spec (loop)
       3. Commit the spec
       4. Plan commits
       5. Implement
       6. Verify against the spec (loop)
          passes → 7, code wrong → 5, spec wrong → 1
       7. Review commits (loop)
          code unchanged → 8, code changed → 6
       8. Push

## Consequences

- My review shifts from reading code to writing specs, reading the
  verification report, skimming structure and tests, and reading the
  code the report points me to.
- Each scenario's verifier says how much to trust it, so pushing
  scenarios from `review` towards `test` and `types` makes the whole
  approach more reliable.
- Verification runs locally and its report stays in the session. It
  can move to CI once the approach has settled.
- Agent judgement is still not deterministic. The deterministic checks
  and my own reading of critical and shared code are what limit the
  risk.
- Where specs live, and how a test file names the spec it belongs to,
  are left open; they depend on the code organisation, which a later
  ADR decides.
