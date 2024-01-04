use log::info;

pub(crate) async fn log_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = &req.method().clone();
    let uri = &req.uri().clone();
    let path = uri.path();

    let start = std::time::Instant::now();

    let response = next.run(req).await;

    let end = std::time::Instant::now();
    let duration = end - start;

    info!(
        "Responding to {} {} with HTTP {} in {}ms",
        method,
        path,
        response.status().as_u16(),
        duration.as_millis()
    );

    response
}
