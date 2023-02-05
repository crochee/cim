use askama::Template;
use axum::response::{Html, IntoResponse, Response};

use cim_core::Error;

pub mod security;
pub mod valid;

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render().map_err(Error::any) {
            Ok(html) => Html(html).into_response(),
            Err(err) => err.into_response(),
        }
    }
}
