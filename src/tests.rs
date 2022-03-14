use std::net::{SocketAddr, TcpListener};
use std::{assert_eq, println};

use axum::body::{Body, HttpBody};
use axum::routing::post;
use axum::{BoxError, Router, Server};
use http::{Request, StatusCode};
use reqwest::RequestBuilder;
use serde::Deserialize;
use tower::make::Shared;
use tower_service::Service;

use crate::Xml;

pub struct TestClient {
    client: reqwest::Client,
    addr: SocketAddr,
}

impl TestClient {
    #[allow(clippy::type_repetition_in_bounds)]
    pub(crate) fn new<S, ResBody>(svc: S) -> Self
    where
        S: Service<Request<Body>, Response = http::Response<ResBody>> + Clone + Send + 'static,
        ResBody: HttpBody + Send + 'static,
        ResBody::Data: Send,
        ResBody::Error: Into<BoxError>,
        S::Future: Send,
        S::Error: Into<BoxError>,
    {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Could not bind ephemeral socket");
        let addr = listener.local_addr().unwrap();
        println!("Listening on {}", addr);

        tokio::spawn(async move {
            let server = Server::from_tcp(listener).unwrap().serve(Shared::new(svc));
            server.await.expect("server error");
        });

        Self {
            client: reqwest::Client::new(),
            addr,
        }
    }

    pub(crate) fn post(&self, url: &str) -> RequestBuilder {
        self.client.post(format!("http://{}{}", self.addr, url))
    }
}

#[tokio::test]
async fn deserialize_body() {
    #[derive(Debug, Deserialize)]
    struct Input {
        foo: String,
    }

    let app = Router::new().route("/", post(|input: Xml<Input>| async { input.0.foo }));

    let client = TestClient::new(app);
    let res = client
        .post("/")
        .body(r#"<Input foo="bar"/>"#)
        .header("content-type", "application/xml")
        .send()
        .await
        .unwrap();
    let body = res.text().await.unwrap();

    assert_eq!(body, "bar");
}

#[tokio::test]
async fn consume_body_to_xml_requires_xml_content_type() {
    #[derive(Debug, Deserialize)]
    struct Input {
        foo: String,
    }

    let app = Router::new().route("/", post(|input: Xml<Input>| async { input.0.foo }));

    let client = TestClient::new(app);
    let res = client
        .post("/")
        .body(r#"<Input foo="bar"/>"#)
        .send()
        .await
        .unwrap();

    let status = res.status();
    assert!(res.text().await.is_ok());

    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn xml_content_types() {
    async fn valid_xml_content_type(content_type: &str) -> bool {
        #[derive(Deserialize)]
        struct Value {}

        println!("testing {:?}", content_type);

        let app = Router::new().route("/", post(|Xml(_): Xml<Value>| async {}));

        let res = TestClient::new(app)
            .post("/")
            .header("content-type", content_type)
            .body("<Value />")
            .send()
            .await
            .unwrap();

        res.status() == StatusCode::OK
    }

    assert!(valid_xml_content_type("application/xml").await);
    assert!(valid_xml_content_type("application/xml; charset=utf-8").await);
    assert!(valid_xml_content_type("application/xml;charset=utf-8").await);
    assert!(valid_xml_content_type("application/cloudevents+xml").await);
    assert!(valid_xml_content_type("text/xml").await);
    assert!(!valid_xml_content_type("application/json").await);
}
