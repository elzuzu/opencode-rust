# Ticket 5: Migrate Data Validation and Parsing

**Task:** Migrate data validation and parsing logic to `serde` and other specialized crates.

**Description:** The application uses several libraries for data validation and parsing, including `zod`, `zod-to-json-schema`, `jsonc-parser`, and `gray-matter`. This ticket involves migrating this functionality to Rust's `serde` framework, along with other specialized crates like `validator`, `json_comments`, and `frontmatter`.

**Acceptance Criteria:**
- All data structures are defined as Rust structs with `serde` annotations.
- Data validation rules are implemented using the `validator` crate.
- JSON with comments can be parsed correctly.
- Files with front matter can be parsed correctly.
- The new data validation and parsing logic is integrated with the main application logic.

**Files to Migrate:**
- All files that use `zod`, `zod-to-json-schema`, `jsonc-parser`, and `gray-matter`.

**Suggested Rust Crates:**
- `serde` (with the `derive` feature)
- `validator` (with the `derive` feature)
- `json_comments`
- `frontmatter`

**Unit Test Examples:**

```rust
// in tests/validation.rs

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use validator::Validate;

    #[derive(Deserialize, Validate)]
    struct User {
        #[validate(length(min = 1))]
        name: String,
        #[validate(email)]
        email: String,
    }

    #[test]
    fn test_valid_user() {
        let user = User {
            name: "test".to_string(),
            email: "test@example.com".to_string(),
        };
        assert!(user.validate().is_ok());
    }

    #[test]
    fn test_invalid_user() {
        let user = User {
            name: "".to_string(),
            email: "test".to_string(),
        };
        assert!(user.validate().is_err());
    }
}
```
