pub mod crypto;
pub mod errors;
mod id;
pub mod regexp;

pub type Result<T, E = errors::WithBacktrace> = core::result::Result<T, E>;

pub use id::next_id;

#[cfg(feature = "axum-resp")]
pub use axum::HtmlTemplate;

#[cfg(feature = "axum-resp")]
mod axum {
    use super::errors;
    use askama::Template;
    use axum::response::{IntoResponse, Response};

    pub struct HtmlTemplate<T>(pub T);

    impl<T> IntoResponse for HtmlTemplate<T>
    where
        T: Template,
    {
        fn into_response(self) -> Response {
            match self.0.render().map_err(errors::any) {
                Ok(html) => axum::response::Html(html).into_response(),
                Err(err) => err.into_response(),
            }
        }
    }
}
