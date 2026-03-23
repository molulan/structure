# Learning Progress Tracker

Track your mastery of concepts as you build the training app. Check off items as you complete them.

## Legend

- [ ] Not started
- [~] In progress
- [x] Completed
- [!] Needs review/reinforcement

---

## Phase 1: Flutter Fundamentals (Weeks 1-2)

### Setup & Tooling
- [ ] Flutter project initialization
- [ ] IDE configuration (VS Code/Zed)
- [ ] flutter_rust_bridge integration setup
- [ ] Hot reload understanding

### Dart Language
- [ ] Variables and types
- [ ] Functions and arrow syntax
- [ ] Classes and constructors
- [ ] Null safety
- [ ] Collections (List, Map)
- [ ] Async/await and Futures
- [ ] Streams

### Flutter Widgets
- [ ] Widget tree concept
- [ ] Stateless vs Stateful widgets
- [ ] Build method and context
- [ ] MaterialApp and Scaffold
- [ ] Common widgets (Text, Container, Row, Column)
- [ ] ListView and ListView.builder
- [ ] Forms and TextField
- [ ] Buttons and interactions
- [ ] Navigation and routing

### Responsive Design
- [ ] MediaQuery and screen size detection
- [ ] LayoutBuilder
- [ ] Breakpoint strategy
- [ ] Mobile-first approach
- [ ] Desktop adaptations

### State Management
- [ ] setState for local state
- [ ] Lifting state up
- [ ] Provider/ChangeNotifier
- [ ] State persistence

**Phase 1 Project Milestones:**
- [ ] Display Mesocycles from Rust in Flutter
- [ ] Basic workout tracking UI
- [ ] Navigation between screens
- [ ] Responsive layout working on phone and desktop

---

## Phase 2: Rust Core & FFI (Weeks 3-4)

### FFI Integration
- [ ] flutter_rust_bridge setup
- [ ] Understanding FFI-safe types
- [ ] Generating Dart bindings
- [ ] Calling Rust from Dart
- [ ] Error handling across FFI boundary

### Rust Refresher (The Book Ch 1-16 review)
- [ ] Ownership and borrowing in practice
- [ ] Structs and enums (advanced patterns)
- [ ] Pattern matching deep dive
- [ ] Option and Result handling
- [ ] Collections (Vec, HashMap)
- [ ] Error handling strategies
- [ ] Testing in Rust

### Advanced Rust (Post-Ch 16)
- [ ] Smart pointers (Box, Rc, Arc)
- [ ] Traits and trait objects
- [ ] Lifetimes in complex scenarios
- [ ] Closures and iterators
- [ ] Module system and visibility

### Database Operations
- [ ] SQLite basics with rusqlite
- [ ] Database schema design
- [ ] CRUD operations
- [ ] Transactions
- [ ] Error handling with SQLite

### Async Rust
- [ ] async/await syntax
- [ ] Futures and executors
- [ ] File I/O operations
- [ ] Concurrency basics

**Phase 2 Project Milestones:**
- [ ] FFI bridge working with existing models
- [ ] SQLite persistence implemented
- [ ] All Exercise and Set types from TODO.md added
- [ ] Tests passing for new functionality

---

## Phase 3: Desktop Builder UI (Weeks 5-6)

### Advanced Flutter
- [ ] Custom widgets
- [ ] Drag and drop (if implemented)
- [ ] Forms with validation
- [ ] Complex layouts
- [ ] Theming and styling

### Responsive Architecture
- [ ] Adaptive layouts
- [ ] Screen size breakpoints
- [ ] Orientation handling
- [ ] Desktop-specific interactions

**Phase 3 Project Milestones:**
- [ ] Program builder interface on desktop
- [ ] Drag-drop exercise ordering (optional)
- [ ] Form-heavy microcycle creation
- [ ] Mobile UX remains excellent

---

## Phase 4: Sync & Integration (Weeks 7-8)

### Data Serialization
- [ ] Serde deep dive
- [ ] JSON serialization
- [ ] Custom serialization logic
- [ ] Versioning and migrations

### File Operations
- [ ] File I/O in Rust
- [ ] File I/O in Flutter
- [ ] Platform-specific paths
- [ ] Error handling for file ops

### Cloud Integration
- [ ] Platform share sheets
- [ ] Cloud storage APIs (iCloud, Google Drive, Dropbox)
- [ ] Sync conflict resolution (basic)

### Testing Strategy
- [ ] Unit tests for business logic
- [ ] Integration tests
- [ ] Widget tests in Flutter
- [ ] FFI testing

**Phase 4 Project Milestones:**
- [ ] JSON export/import working
- [ ] File sharing between desktop and mobile
- [ ] Cloud storage integration (at least one provider)
- [ ] Complete workout tracking flow

---

## Ongoing Concepts

### Architecture & Design
- [ ] Separation of concerns
- [ ] Repository pattern
- [ ] Clean architecture principles
- [ ] Error handling strategy
- [ ] Logging and debugging

### Development Practices
- [ ] Git workflow
- [ ] Code review practices
- [ ] Documentation
- [ ] Refactoring techniques

### Performance
- [ ] Profiling Flutter apps
- [ ] Rust optimization basics
- [ ] Database query optimization
- [ ] Memory management

---

## Daily Log Template

```markdown
## YYYY-MM-DD

**Today's Focus:** [What phase/concept]

**Time Spent:** [X hours]

**What I Learned:**
- [Concept 1]
- [Concept 2]

**Challenges:**
- [Challenge and how solved]

**Code Written:**
- [Files/locations]

**Questions for Tomorrow:**
- [Question 1]
- [Question 2]

**Mood/Energy:** [1-10]
```

---

## Weekly Review Template

**Week of:** [Date]

**Progress Summary:**
- Phase X: [% complete]
- Concepts mastered: [List]
- Lines of code written: [Approximate]

**What Went Well:**
- [Success 1]
- [Success 2]

**What Was Challenging:**
- [Challenge 1]
- [Challenge 2]

**Adjustments for Next Week:**
- [Adjustment 1]

**Celebration:** [What you're proud of]
