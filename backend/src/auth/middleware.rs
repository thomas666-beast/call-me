use crate::auth::jwt::{verify_access_token, Claims};
use crate::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

#[derive(Clone, Debug)]
pub struct AuthUser(pub Claims);

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Try the Authorization: Bearer <token> header.
    let token_from_header = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer ").map(|s| s.to_string()));

    // 2. Fallback: ?token=<jwt> in the query string. Used by browser
    //    <audio> tags, which cannot send custom headers.
    //
    //    SECURITY NOTE: query-string tokens can end up in server logs and
    //    browser history. Acceptable for dev. In production, replace this
    //    with short-lived signed download URLs.
    let token = match token_from_header {
        Some(t) => t,
        None => {
            let query = req.uri().query().unwrap_or("");
            let mut found = None;
            for pair in query.split('&') {
                if let Some(value) = pair.strip_prefix("token=") {
                    found = Some(value.to_string());
                    break;
                }
            }
            found.ok_or(StatusCode::UNAUTHORIZED)?
        }
    };

    let claims = verify_access_token(&state.cfg.jwt_secret, &token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(AuthUser(claims));
    Ok(next.run(req).await)
}
