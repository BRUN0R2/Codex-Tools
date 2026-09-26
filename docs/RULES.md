# Project Constitution

## Supreme Rule

Codex / AI must follow all project rules rigorously and without exception.

No implementation, optimization, abstraction, refactoring, dependency, shortcut, or architectural decision may violate the rules defined in this document.

When there is a conflict, always use this priority order:

1. Project rules
2. Architectural integrity
3. Maintainability
4. Predictability
5. Security
6. Performance
7. Development speed

Speed never justifies architectural degradation.

---

## High-Priority Modularity Rules

Modularity is mandatory at two levels, and both have very high priority.

### Folders and files

* Folder and file organization must be modular.
* Do not create new code in generic or accumulated folders without a clear responsibility.
* The directory structure must reflect real, small, cohesive, and predictable modules.
* Each folder must have an explicit purpose and contain only directly related files.
* When the structure is ambiguous, modularize before adding features.

### Code

* The code inside each module must also be modular, cohesive, and explicit.
* Each class, function, state, and contract must have one clear responsibility.
* Do not concentrate multiple responsibilities in one implementation for convenience.
* When a function, class, or file begins to accumulate different rules, separate the module before adding features.

---

# Core Philosophy

The project must remain:

* Clean
* Predictable
* Minimal
* Modular
* Explicit
* Maintainable
* Scalable
* Production-ready

The codebase must continuously evolve toward simplicity, never toward complexity.

Every implementation must solve real problems with the least complexity necessary.

Avoid:

* Overengineering
* Premature abstractions
* Temporary solutions
* Hidden behavior
* Implicit flows
* Defensive chaos
* Architectural inconsistency

The system must remain understandable months later without relying on historical context.

---

# Repository Rules

* Maintain a local Git repository from the start.
* Commit each logically complete and validated change.
* Commit messages must:

  * be in English
  * be short
  * use an imperative verb
  * clearly describe the actual changes
* Keep `docs/TODO.md` minimal, current, and actionable.
* Never commit secrets, credentials, tokens, or private data.
* Avoid files, dependencies, assets, logs, or tools without a real need.
* Prefer a clean, working codebase over preserving obsolete compatibility.
* Avoid deliberately accumulating technical debt.

---

# Architecture Rules

## Structure

* Organize systems into small, cohesive modules.
* Each module must have a clear responsibility.
* Prefer composition over inheritance.
* Prefer explicit contracts over implicit behavior.
* Prefer deterministic flows over dynamic “magic.”
* Prefer simple abstractions over deep abstraction layers.
* Avoid oversized objects and centralized accumulations of logic.
* Business rules must not spread unpredictably.

## Design

* Keep APIs small and explicit.
* Make inputs, outputs, side effects, and failures visible.
* Do not use silent fallbacks.
* Do not hide startup recovery paths.
* Do not use false resilience to hide real failures.
* Report errors clearly and predictably.
* Make state transitions traceable.

## Evolution

* Prepare systems for future expansion without destructive rewrites.
* Refactoring must simplify the project, not reorganize complexity.
* Reduce fragmentation whenever possible.
* Continuously remove dead, obsolete, or duplicated code.

---

# Coding Rules

## Style

* Use modern Rust for the Tauri backend and modern TypeScript for the frontend.
* Use native Windows APIs only in isolated, explicit, and small modules.
* Prefer modern, secure, stable versions supported by the current toolchain.
* Keep TypeScript in strict mode and Rust free of relevant warnings.
* Keep code semantic, clean, direct, and optimized.
* Prioritize readability over cleverness.
* Use strong typing whenever possible.
* Avoid ambiguous names.
* Avoid unnecessary indirection.
* Avoid excessive complexity with generics, traits, or conditional types without justification.

## Naming

* Use clear, descriptive names.
* Avoid abbreviations unless they are universally understood.
* Avoid artificial prefixes and suffixes.
* Internal TypeScript files must use PascalCase without separators.
* Rust modules must follow the toolchain's snake_case conventions.
* Use snake_case in TypeScript only when an external requirement calls for it.

## Logic

Apply:

* Single Responsibility Principle
* DRY
* Explicit ownership
* Explicit lifetime management

Avoid:

* Magic numbers
* Dead code
* Obsolete commented-out code
* Hidden state mutation
* Implicit ownership
* Unsafe resource handling

Constants must always have:

* Semantic meaning
* An explicit type
* Clear context

---

# Runtime and Reliability Rules

* Runtime validation is the primary source of confidence.
* Prefer live validation over an excess of automated tests.
* Create tests only when they deliver real, measurable value.
* Avoid noisy, redundant, or expensive-to-maintain tests.
* Keep logs only when they are operationally useful.
* Avoid debug noise.

The system must continuously validate:

* Memory safety
* Resource lifetimes
* Ownership correctness
* Initialization order
* Failure visibility

---

# Performance Rules

* Optimize responsibly.
* Never sacrifice maintainability for micro-optimizations.
* Avoid unnecessary allocations.
* Avoid unnecessary runtime overhead.
* Prioritize stable, predictable performance.
* Measure before aggressive optimization.

Performance must be intentional, never accidental.

---

# Dependency Rules

* Every dependency must justify its existence.
* Prefer internal solutions when complexity is low.
* Avoid excessive dependencies.
* Isolate external integrations.
* Regularly update dependencies to modern, secure versions.

---

# UI and UX Rules

* Keep interfaces clean, direct, and functional.
* Do not add visual complexity without practical value.
* Avoid states, options, or controls without real utility.
* Keep menus and flows low-friction.
* Keep information density organized and intentional.

---

# AI Operational Rules

AI must:

* Think before implementing.
* Preserve architectural consistency.
* Detect future maintenance risks.
* Flag architectural violations before proceeding.
* Avoid speculative implementations.
* Never invent APIs, systems, or behavior that do not exist.
* Avoid partial and unfinished solutions.
* Prefer complete, working implementations.

Before finalizing any change, always review:

* Duplication
* Dead code
* Ambiguity
* Unsafe ownership
* Maintenance impact
* Architectural consistency

---

# Prohibited Patterns

Explicitly avoid:

* Singleton abuse
* Service Locator abuse
* Hidden globals
* Circular dependencies
* Deep inheritance trees
* State mutation without clear ownership
* God classes
* Runtime reflection abuse
* Implicit resource ownership

---

# Engineering Decision Rules

When multiple solutions are available, always prefer the one that:

1. Reduces future maintenance
2. Improves predictability
3. Reduces hidden complexity
4. Minimizes coupling
5. Makes debugging easier
6. Has fewer moving parts
7. Preserves architectural consistency

---

# Final Directive

Interpret all future instructions through this constitution.

If a request conflicts with these rules, explicitly report the conflict before continuing implementation.

Long-term integrity of the project is mandatory and non-negotiable.
