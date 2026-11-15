---
name: comment-style
description: >
  How to write and review code comments and doc comments in this repo, in Rust
  and TypeScript/React. Use when adding or changing a comment or doc comment,
  adding a public item or module, reviewing the comments in a diff, or deciding
  whether code needs a comment or a clearer name, type or structure instead.
allowed-tools: Read Edit Grep Glob Bash(cargo:*) Bash(npm:*)
---

# Comment style

There are two kinds of comment, written for different readers:

- **Comments in the code** are read by someone working on the implementation.
  They explain why the code exists.
- **Doc comments** are read by someone using the item, often in rendered docs or
  an editor tooltip, without the body in front of them. They state what the item
  is or does and what it guarantees.

## Comments in the code

The standard is Google's code review guidance on comments
(https://google.github.io/eng-practices/review/reviewer/looking-for.html#comments):
every comment is necessary. A comment explains why some code exists, not what
it is doing. Regular expressions and complex algorithms are the exceptions:
there, saying what the code does helps. Mostly, a comment carries what the code can't,
like the reasoning behind a decision.

Change the code before commenting it:

- If the code isn't clear enough to explain itself, make it simpler: a clearer
  name, a smaller function or a simpler structure.
- If a comment explains a workaround or an unwritten rule, look for a type or a
  name that makes it hold by itself (see "Make illegal states unrepresentable"
  in CLAUDE.md). A comment is the fallback when the code can't carry it.
- Keep it to one or two lines. A comment that needs a paragraph usually means
  the code should change.

## Doc comments

The standard follows Google's Java style guide on Javadoc
(https://google.github.io/styleguide/javaguide.html#s7.3.1-javadoc-exception-self-explanatory).

- Every public item gets a doc comment: types, functions, fields, enum variants,
  constants, and exported TypeScript functions, components, hooks, types and
  props.
- The exception is an item that is truly self-explanatory, where there is
  nothing to say beyond its name, like a plain getter, `new`, or a field named by
  what it holds. Don't use the exception to leave out something a reader new to
  the code would need.
- A name that carries a domain term is never self-explanatory: mesocycle,
  microcycle, set group, intensity, RIR, a grid cell. Say what the term means
  here.
- Always document the boundaries: the public API of `structure-core` that the
  server and FFI call, and the `#[ts(export)]` wire types. A wire type's doc is
  copied into `web/src/api/generated/`, so write it for a TypeScript reader with
  no Rust context.
- Every module gets a doc comment saying what it owns.
- Open with a one-sentence summary of what the item is or does. Then add only
  what the caller can't see from the signature: units, invariants, ordering,
  side effects, when it fails. Don't repeat the types.
- Explain why only when it matters to the caller.
- Refer to other items with a link the tooling checks, not a bare name.
- Examples are rare. Behaviour is covered by the tests; add an example only when
  how to use the item isn't obvious.
- There is no `missing_docs` lint: it can't tell a self-explanatory item from
  one that needs a doc.

## Writing either kind

- Write in clear, plain English: full sentences that start with the subject,
  are capitalised and end with a period.
- Describe the code as it stands. Don't mention the change that produced it,
  what it replaced, or what it may become. A reader months from now never saw
  the previous state. A change is explained in its commit message, and future
  work goes in an issue. Naming an alternative that was ruled out, and why, is
  not history: "uses X rather than Y, because Y can't Z" explains the code.
- When a change makes a comment wrong, rewrite the comment to say what the code
  does now. Don't add a contrast with what it used to do.

## Rust

- Comments in the code use `//`. Doc comments use `///` on an item and `//!` at
  the top of a module. Never `/* */` or `/** */`.
- Link with intra-doc links: `` [`SetGroup`] ``. `cargo doc` checks them.
- Add `# Errors`, `# Panics` or `# Safety` only when a caller can't tell from the
  signature and the error type.

## TypeScript and React

- Comments in the code use `//`. Doc comments use `/** */` TSDoc, on exported
  items and at the top of a module file.
- Link with `{@link Name}`.

## Reviewing comments in a diff

- Only touch comments the diff adds or makes wrong. Leave older comments alone,
  even ones you'd word differently.
- Check that every claim is true, by reading the code or by running it (`cargo
  test`, `cargo doc`, `npm run typecheck`). A well-worded comment can still be
  wrong.

### Comments in the code

- Hold each one against "Comments in the code", "Writing either kind" and the
  section for its language.
- When a comment isn't necessary, delete it rather than rewording it.

### Doc comments

- Hold each one, and each public item and module the diff adds, against "Doc
  comments", "Writing either kind" and the section for its language. A missing
  doc comment is a finding.
