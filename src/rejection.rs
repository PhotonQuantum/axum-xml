use axum_core::body;
use axum_core::extract::rejection::BytesRejection;
use axum_core::response::Response;
use http::StatusCode;
use http_body::Full;
use thiserror::Error;

use crate::IntoResponse;

#[derive(Debug, Error)]
pub enum XmlRejection {
    #[error("Failed to parse the request body as XML")]
    InvalidXMLBody(#[from] quick_xml::DeError),
    #[error("Expected request with `Content-Type: application/xml`")]
    MissingXMLContentType,
    #[error("{0}")]
    BytesRejection(#[from] BytesRejection),
}

impl IntoResponse for XmlRejection {
    fn into_response(self) -> crate::Response {
        match self {
            e @ XmlRejection::InvalidXMLBody(_) => {
                let mut res = Response::new(body::boxed(Full::from(format!("{}", e))));
                *res.status_mut() = StatusCode::UNPROCESSABLE_ENTITY;
                res
            }
            e @ XmlRejection::MissingXMLContentType => {
                let mut res = Response::new(body::boxed(Full::from(format!("{}", e))));
                *res.status_mut() = StatusCode::UNSUPPORTED_MEDIA_TYPE;
                res
            }
            XmlRejection::BytesRejection(e) => e.into_response(),
        }
    }
}
