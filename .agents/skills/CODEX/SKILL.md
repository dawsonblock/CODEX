```markdown
# CODEX Development Patterns

> Auto-generated skill from repository analysis

## Overview
This skill teaches you the core development patterns and conventions used in the CODEX Rust repository. You'll learn about file naming, import/export styles, commit message habits, and how to structure and run tests. This guide is ideal for contributors looking to maintain consistency and quality in the CODEX codebase.

## Coding Conventions

### File Naming
- **Style:** snake_case
- **Example:**  
  ```plaintext
  user_profile.rs
  data_manager.rs
  ```

### Import Style
- **Style:** Relative imports
- **Example:**  
  ```rust
  mod utils;
  use crate::helpers::math;
  ```

### Export Style
- **Style:** Named exports
- **Example:**  
  ```rust
  pub fn calculate_sum(a: i32, b: i32) -> i32 {
      a + b
  }
  ```

### Commit Messages
- **Style:** Freeform, no strict prefixes
- **Average Length:** ~45 characters
- **Examples:**  
  ```
  fix bug in data parser
  add new user authentication flow
  ```

## Workflows

### Adding a New Module
**Trigger:** When you need to add a new logical component or feature.
**Command:** `/add-module`

1. Create a new `.rs` file using snake_case naming.
2. Implement your logic, using relative imports for dependencies.
3. Export functions or structs using `pub`.
4. Add relevant tests in a corresponding `*_test.rs` file.
5. Commit changes with a descriptive message.

### Writing and Running Tests
**Trigger:** When you need to verify the correctness of your code.
**Command:** `/run-tests`

1. Create test files matching the pattern `*.test.*` (e.g., `math.test.rs`).
2. Write tests using Rust's built-in test framework.
3. Run tests using Cargo:
   ```sh
   cargo test
   ```
4. Review test results and fix any failing cases.

## Testing Patterns

- **Framework:** Unknown (likely Rust's built-in test framework)
- **File Pattern:** `*.test.*` (e.g., `user.test.rs`)
- **Example Test:**
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn test_calculate_sum() {
          assert_eq!(calculate_sum(2, 3), 5);
      }
  }
  ```

## Commands
| Command       | Purpose                                      |
|---------------|----------------------------------------------|
| /add-module   | Scaffold and implement a new Rust module     |
| /run-tests    | Run all tests in the repository              |
```
