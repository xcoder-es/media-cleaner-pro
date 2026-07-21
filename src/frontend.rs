use axum::{
    extract::Request,
    http::header,
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Assets;

pub async fn serve_assets(request: Request) -> Response {
    let path = request.uri().path().trim_start_matches('/').to_string();

    let path = if path.is_empty() { "index.html" } else { &path };

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            // SPA fallback — serve index.html for client-side routing
            match Assets::get("index.html") {
                Some(content) => {
                    ([(header::CONTENT_TYPE, "text/html")], content.data).into_response()
                }
                None => (
                    [(header::CONTENT_TYPE, "text/plain")],
                    "Frontend not found. Build with: cd frontend && npm install && npm run build"
                        .as_bytes()
                        .to_vec(),
                )
                    .into_response(),
            }
        }
    }
}
