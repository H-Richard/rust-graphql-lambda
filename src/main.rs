use async_graphql::{EmptySubscription, Schema};
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use lambda_http::{
    http::{Method, StatusCode},
    run, service_fn, tracing, Body, Error, Request, Response,
};
use schema::{Mutation, Query};

mod schema;

fn request_error(code: StatusCode, body: Option<String>) -> Result<Response<Body>, Error> {
    let body = match body {
        Some(v) => Body::Text(v),
        None => Body::Empty,
    };
    Ok(Response::builder().status(code).body(body)?)
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    if event.method() != Method::POST {
        return request_error(StatusCode::METHOD_NOT_ALLOWED, Option::None);
    }

    let body = match event.clone().into_body() {
        Body::Empty => return request_error(StatusCode::UNPROCESSABLE_ENTITY, Option::None),
        Body::Text(text) => serde_json::from_str::<async_graphql::Request>(&text),
        Body::Binary(bin) => serde_json::from_slice(&bin),
    };

    let query = match body {
        Ok(query) => query,
        Err(e) => return request_error(StatusCode::BAD_REQUEST, Option::Some(e.to_string())),
    };

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let db = Client::new(&config);
    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(db)
        .finish();

    let response_body = serde_json::to_string(&schema.execute(query).await)?;
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::Text(response_body))?;

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
