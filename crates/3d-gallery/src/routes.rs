//! Route handlers for 3D Gallery viewer

use crate::models::{ErrorResponse, GalleryQuery};
use askama::Template;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

/// Main 3D viewer page template
#[derive(Template)]
#[template(path = "viewer.html")]
struct ViewerTemplate {
    access_code: String,
}

/// Error page template
#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    error: ErrorResponse,
}

/// Handler for /3d and /digital-twin routes
///
/// Renders the 3D gallery viewer page.
/// Requires an access code in the query string.
///
/// # Example
/// GET /3d?code=abc123xyz
pub async fn viewer_page(Query(query): Query<GalleryQuery>) -> Response {
    // For now, just render the template
    // Access code validation will happen in the frontend via API call
    let template = ViewerTemplate {
        access_code: query.code,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            tracing::error!("Template render error: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to render template",
            )
                .into_response()
        }
    }
}

/// Handler for error pages
///
/// Renders an error page with appropriate message
pub async fn error_page(error: ErrorResponse) -> Response {
    let template = ErrorTemplate { error };

    match template.render() {
        Ok(html) => (StatusCode::BAD_REQUEST, Html(html)).into_response(),
        Err(err) => {
            tracing::error!("Error template render failed: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "An error occurred").into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewer_template_creation() {
        let template = ViewerTemplate {
            access_code: "test123".to_string(),
        };
        assert_eq!(template.access_code, "test123");
    }

    #[test]
    fn test_error_response_invalid_code() {
        let err = ErrorResponse::invalid_code();
        assert_eq!(err.code, "INVALID_CODE");
    }
}
