# `axum` integration

`su_auth` already comes with `axum` integration right out of the box.

If you want to gate a particular endpoint handler function behind a verified user, you only need to use the `VerifiedUser` extractor:

```rust
use axum::response::Html;
use su_auth::user::{UnverifiedUser, User, VerifiedUser};

// Only for verified users
async fn get_orders(user: VerifiedUser) -> Html<String> {
    Html(format!("hello user ID #{}", user.user_id()))
}

// Only for unverified users
async fn verify_page(user: UnverifiedUser) -> Html<String> {
    Html(format!("hello unverified user ID #{}", user.user_id()))
}

// For...banned?
async fn shame(user: BannedUser) -> Html<String> {
    Html(format!("hello banned user ID #{} :P", user.user_id()))
}

// For all!
async fn landing(user: Option<User>) -> Html<String> {
    Html(
        format!(
            "hello {}",
            user.map_or("guest".to_string(), |u| u.user_id().to_string())
        )
    )
}
```
