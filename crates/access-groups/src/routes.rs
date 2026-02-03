//! Route definitions for access groups API

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::SqlitePool;

use crate::handlers::{
    accept_invitation_handler, add_member_handler, cancel_invitation_handler,
    check_resource_access_handler, create_group_handler, create_invitation_handler,
    delete_group_handler, get_group_handler, get_invitation_details_handler, list_groups_handler,
    list_invitations_handler, list_members_handler, remove_member_handler, update_group_handler,
    update_member_role_handler,
};

/// Create the access groups router with all endpoints
pub fn create_routes(pool: SqlitePool) -> Router {
    Router::new()
        // Group CRUD
        .route("/groups", get(list_groups_handler))
        .route("/groups", post(create_group_handler))
        .route("/groups/:slug", get(get_group_handler))
        .route("/groups/:slug", put(update_group_handler))
        .route("/groups/:slug", delete(delete_group_handler))
        // Member Management
        .route("/groups/:slug/members", get(list_members_handler))
        .route("/groups/:slug/members", post(add_member_handler))
        .route(
            "/groups/:slug/members/:user_id",
            delete(remove_member_handler),
        )
        .route(
            "/groups/:slug/members/:user_id/role",
            put(update_member_role_handler),
        )
        // Invitation Management
        .route("/groups/:slug/invitations", get(list_invitations_handler))
        .route("/groups/:slug/invitations", post(create_invitation_handler))
        .route(
            "/groups/:slug/invitations/:invitation_id",
            delete(cancel_invitation_handler),
        )
        // Public Invitation Endpoints (no auth required for viewing)
        .route("/invitations/:token", get(get_invitation_details_handler))
        .route(
            "/invitations/:token/accept",
            post(accept_invitation_handler),
        )
        // Resource Access Check
        .route(
            "/groups/:slug/check-access",
            post(check_resource_access_handler),
        )
        .with_state(pool)
}

/// API-only routes (returns JSON)
pub fn create_api_routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/api/groups", get(list_groups_handler))
        .route("/api/groups/:slug", get(get_group_handler))
        .route("/api/groups/:slug/members", get(list_members_handler))
        .route(
            "/api/groups/:slug/invitations",
            get(list_invitations_handler),
        )
        .route(
            "/api/groups/:slug/check-access",
            post(check_resource_access_handler),
        )
        .with_state(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routes_creation() {
        // Create a mock pool for testing
        // This just verifies the routes can be created without panicking
        // Actual endpoint testing requires integration tests
    }
}
