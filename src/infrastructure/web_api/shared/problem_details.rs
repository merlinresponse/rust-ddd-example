use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::domain::shared::{DomainError, Error};

/// ApiError — turns any use-case error into an RFC 9457 "Problem Details" JSON response.
///
/// Why modelled this way: the ONE place that maps business errors to HTTP status codes, by
/// matching on the error *variant*. Controllers just use `?`.
pub struct ApiError(pub Error);

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // FLOW 9b — WHAT: choose the HTTP status from the error *variant*.
        // WHY: matching on a type is checked by the compiler; matching on message text breaks
        // the day someone rewords a message.
        let (status, title, detail) = match &self.0 {
            Error::Domain(error) => {
                let (status, title) = match error {
                    DomainError::TaskNotFound(_) => (StatusCode::NOT_FOUND, "Task not found"),
                    DomainError::ChecklistItemNotFound(_) => {
                        (StatusCode::NOT_FOUND, "Checklist item not found")
                    }
                    DomainError::InvalidStatusTransition { .. }
                    | DomainError::CompletedTaskCannotBeChanged
                    | DomainError::TaskHasOpenChecklistItems { .. } => {
                        (StatusCode::CONFLICT, "Business rule violated")
                    }
                    DomainError::InvalidTaskId(_)
                    | DomainError::EmptyTaskDescription
                    | DomainError::TaskDescriptionTooLong { .. }
                    | DomainError::UnknownTaskStatus(_)
                    | DomainError::InvalidChecklistItemId(_)
                    | DomainError::EmptyChecklistItemTitle
                    | DomainError::ChecklistItemTitleTooLong { .. } => {
                        (StatusCode::BAD_REQUEST, "Invalid input")
                    }
                };
                (status, title, error.to_string())
            }
            Error::Infrastructure(message) => {
                // Log the technical detail, but never leak it to the client.
                eprintln!("[error] {message}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal error",
                    "Something went wrong.".into(),
                )
            }
        };
        let body = json!({
            "type": "about:blank",
            "title": title,
            "status": status.as_u16(),
            "detail": detail,
        });
        (
            status,
            [(header::CONTENT_TYPE, "application/problem+json")],
            body.to_string(),
        )
            .into_response()
    }
}
