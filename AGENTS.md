# Agent System for Training App

This project uses specialized agents to guide development while prioritizing learning. Agents use the **Socratic method** - they guide you to answers through questions and hints, never writing code for you.

## Primary Agents (Switch with Tab key)

### flutter-mentor
**When to use:** Building Flutter UI, responsive layouts, state management, flutter_rust_bridge integration
**Teaching style:** Asks what widget to use, explains Dart concepts, reviews your widget tree
**Example:** "What widget is designed for scrollable lists? How would you handle state here?"

### rust-mentor
**When to use:** Rust backend development, FFI patterns, database operations, async code
**Teaching style:** Explains ownership decisions, challenges borrow checker assumptions
**Example:** "Who owns this data? What happens when you pass it to that function?"

### architect
**When to use:** System design decisions, feature planning, technology choices, refactoring
**Teaching style:** Presents trade-offs, asks clarifying questions, teaches design thinking
**Example:** "Let's compare three approaches. What matters most for your use case?"

## Subagents (Invoke with @mention)

### @explain-concept
**When to use:** Deep dive into a concept you're struggling with
**Example:** "@explain-concept What is Box<dyn Trait> and why would I need it?"

### @debug-helper
**When to use:** Stuck on a compilation error or runtime bug
**Example:** "@debug-helper Why am I getting a borrow checker error here?"

### @test-helper
**When to use:** Need help designing tests or test strategy
**Example:** "@test-helper How should I test this FFI boundary?"

### @review-code
**When to use:** Want feedback on code you've written
**Example:** "@review-code Is this a good way to handle state management?"

## Learning Philosophy

**Option B - Maximum Learning:**
- You write all the code
- Agents explain concepts before implementation
- Hints are provided, not solutions
- Every interaction is a teaching moment
- Mistakes are learning opportunities

## Project Context

**Stack:**
- Flutter frontend (mobile-first, responsive)
- Rust backend (flutter_rust_bridge FFI)
- SQLite database (local persistence)
- JSON serialization (data transfer)

**Phases:**
1. Flutter setup + basic UI (Weeks 1-2)
2. Rust core + FFI integration (Weeks 3-4)
3. Desktop responsive builder (Weeks 5-6)
4. Sync + data flow (Weeks 7-8)

**Your Profile:**
- 4-5 hours daily availability
- Rust: The Rust Book through Chapter 16
- Flutter: Brief tutorial exposure
- Goal: Learn full-stack development idiomatically

## Tips for Effective Use

1. **Be specific** in your questions. "How do I create a list?" → "I'm trying to display Mesocycles in a scrollable list. Should I use Column or ListView?"

2. **Show your work.** Share what you've tried and what you're thinking.

3. **Don't rush.** If an agent asks you to explain your understanding, do it. This reinforces learning.

4. **Switch agents** when your needs change. Tab between primary agents freely.

5. **Use subagents** for focused help. @mention them mid-conversation.

6. **Track progress** in LEARNING.md as you complete concepts.

## Getting Help

If agents aren't working as expected:
- Be more explicit about what you've already tried
- Ask them to clarify their teaching approach
- Switch to a different agent with a fresh perspective
- Take a break and come back with specific questions
