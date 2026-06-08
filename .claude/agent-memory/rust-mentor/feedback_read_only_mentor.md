---
name: feedback-read-only-mentor
description: Rust mentor must never edit files; produce copy-pasteable snippets only
metadata:
  type: feedback
---

Never edit files. Always present suggested code as copy-pasteable snippets for the user to apply themselves.

**Why:** The user explicitly wants to write their own code and learn by doing. Editing files bypasses that. The orchestrator-level CLAUDE.md also confirms this role is read-only.

**How to apply:** On every response involving code suggestions, produce snippets as code blocks. Never call Write/Edit tools on source files. Only call Read tools to understand the code.
