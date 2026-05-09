```markdown
# CODEX Development Patterns

> Auto-generated skill from repository analysis

## Overview

This skill introduces the core development patterns and workflows used in the CODEX repository, a Rust codebase with no detected framework. You'll learn about its file organization, coding conventions, commit practices, and the step-by-step processes for adding features, updating documentation, and maintaining tests. This guide is ideal for contributors aiming to align with the project's established standards.

## Coding Conventions

- **File Naming:**  
  Use `camelCase` for file names.  
  _Example:_  
  ```
  myFeature.rs
  testHelpers.rs
  ```

- **Import Style:**  
  Use **relative imports** within modules.  
  _Example:_  
  ```rust
  // In src/myFeature.rs
  use super::helperFunctions;
  ```

- **Export Style:**  
  Use **named exports** for modules and functions.  
  _Example:_  
  ```rust
  pub fn my_exported_function() { ... }
  ```

- **Commit Messages:**  
  Follow the **conventional commit** style with prefixes like `feat` and `docs`.  
  _Example:_  
  ```
  feat: add workspace isolation for concurrent builds
  docs: update architecture diagram for new scheduler
  ```

## Workflows

### Feature Implementation with Tests and Docs
**Trigger:** When adding a new capability or major feature  
**Command:** `/new-feature`

1. **Implement feature logic**  
   Add or update Rust source files in the relevant `src/` directory.  
   _Example:_  
   ```
   global-workspace-runtime-rs/crates/myCrate/src/newFeature.rs
   ```

2. **Add or update tests**  
   Create or modify test files in the corresponding `tests/` directory.  
   _Example:_  
   ```
   global-workspace-runtime-rs/crates/myCrate/tests/newFeature.test.rs
   ```

3. **Update documentation**  
   Edit documentation files to reflect the new feature or changes.  
   _Example:_  
   ```
   docs/ARCHITECTURE.md
   docs/LIMITATIONS.md
   ```

4. **Commit changes**  
   Use a conventional commit message, e.g.:  
   ```
   feat: implement new feature for workspace management
   ```

### Test and Docs Minor Update
**Trigger:** When clarifying implementation details, magic numbers, or test setup  
**Command:** `/clarify-tests-docs`

1. **Edit code comments or inline documentation**  
   Update comments in `src/` files for clarity.  
   _Example:_  
   ```rust
   // src/myFeature.rs
   // Clarifies why a timeout is set to 500ms
   ```

2. **Update or clarify test logic**  
   Adjust test files as needed.  
   _Example:_  
   ```
   global-workspace-runtime-rs/crates/myCrate/tests/myFeature.test.rs
   ```

3. **Optionally update documentation**  
   Make clarifications in Markdown docs if necessary.  
   _Example:_  
   ```
   docs/ARCHITECTURE.md
   ```

4. **Commit changes**  
   Use a conventional commit message, e.g.:  
   ```
   docs: clarify test setup for workspace isolation
   ```

## Testing Patterns

- **Test File Naming:**  
  Test files use the pattern `*.test.rs`.  
  _Example:_  
  ```
  myFeature.test.rs
  ```

- **Test Placement:**  
  Tests reside in the `tests/` directory within each crate.  
  _Example:_  
  ```
  global-workspace-runtime-rs/crates/myCrate/tests/
  ```

- **Test Framework:**  
  The specific test framework is not specified, but standard Rust testing conventions apply.  
  _Example:_  
  ```rust
  #[test]
  fn test_my_feature() {
      // test logic here
  }
  ```

## Commands

| Command           | Purpose                                                        |
|-------------------|----------------------------------------------------------------|
| /new-feature      | Start a new feature implementation with tests and documentation |
| /clarify-tests-docs | Make minor clarifications to tests and documentation         |
```