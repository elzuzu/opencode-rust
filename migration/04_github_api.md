# Ticket 4: Migrate GitHub API Client to Octocrab

**Task:** Migrate the GitHub API client from `@octokit/rest` and `@octokit/graphql` to `octocrab`.

**Description:** The application interacts with the GitHub API using `@octokit/rest` and `@octokit/graphql`. This ticket involves migrating this functionality to `octocrab`, a modern and ergonomic GitHub API client for Rust. The goal is to replicate the existing API interactions, including authentication and data fetching.

**Acceptance Criteria:**
- The application can authenticate with the GitHub API.
- All existing API calls (REST and GraphQL) are migrated to `octocrab`.
- The application can correctly handle responses from the GitHub API.
- The GitHub API client is integrated with the main application logic.

**Files to Migrate:**
- `packages/opencode/src/github/*.ts`

**Suggested Rust Crates:**
- `octocrab`
- `tokio` (for the async runtime)
- `serde` (for JSON serialization/deserialization)

**Unit Test Examples:**

```rust
// in tests/github.rs

#[cfg(test)]
mod tests {
    use super::*;
    use octocrab::Octocrab;

    #[tokio::test]
    async fn test_get_user() {
        let octocrab = Octocrab::builder().build().unwrap();

        let user = octocrab.users("octocat").get().await;
        assert!(user.is_ok());
        assert_eq!(user.unwrap().login, "octocat");
    }
}
```
