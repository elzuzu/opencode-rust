# Ticket 2: Migrate HTTP Server to Axum

**Task:** Migrate the HTTP server from `hono` to `axum`.

**Description:** The existing HTTP server is implemented using `hono`. This ticket involves migrating the server to `axum`, a modern and ergonomic web framework for Rust. The goal is to replicate the existing API endpoints and middleware logic.

**Acceptance Criteria:**
- All existing API routes are implemented in `axum`.
- All middleware for logging, authentication, and error handling is migrated.
- The server can handle requests and responses correctly.
- The server is integrated with the main application logic.

**Files to Migrate:**
- `packages/opencode/src/server/index.ts`
- `packages/opencode/src/server/routes/*.ts`

**Suggested Rust Crates:**
- `axum`
- `tokio` (for the async runtime)
- `serde` (for JSON serialization/deserialization)
- `tower-http` (for middleware)

**Unit Test Examples:**

```rust
// in tests/server.rs

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn test_hello_world() {
        let app = create_app(); // Assuming you have a function to create the app

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"Hello, World!");
    }
}
```
