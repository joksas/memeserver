use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{extract::DefaultBodyLimit, routing::post, Router};
use bytesize::{ByteSize, MB};
use html_to_string_macro::html;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/upload", post(upload))
        .layer(DefaultBodyLimit::disable());

    let port = 8080;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Listening on http://localhost:{port}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> impl IntoResponse {
    let title = "Home";
    Html(html! {
        <html>
            <head>
                <title>{title}</title>
                <script src="https://unpkg.com/htmx.org@1.9.4"></script>
                <script src="https://cdn.tailwindcss.com"></script>
            </head>
            <body>
                <main class="container mx-auto max-w-xl">
                    <h1 class="text-3xl font-bold mb-3">{title}</h1>
                    <h2 class="text-xl font-bold mb-3">"Upload file"</h2>
                    <form
                        class="grid grid-cols-1 gap-2 w-1/2"
                        hx-encoding="multipart/form-data"
                        hx-post="/upload"
                        _="on htmx:xhr:progress(loaded, total) set #progress.value to (loaded/total)*100"
                        >
                        <input type="file" name="file" required />
                        <button class="bg-blue-500 text-white p-2">"Upload"</button>
                    </form>
                </main>
                <script>
                    "document.body.addEventListener('htmx:responseError', (event) => {
                        event.preventDefault();
                        console.log(event);
                        if (event.detail.xhr.responseText) {
                            event.detail.target.innerHTML = event.detail.xhr.responseText;
                        } else {
                            event.detail.target.innerHTML = event.detail.xhr.statusText;
                        }
                    });"
                </script>
            </body>
        </html>
    })
}

async fn upload(mut multipart: Multipart) -> (StatusCode, impl IntoResponse) {
    const MAX_SIZE: u64 = 10 * MB;

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();

        let mut total = 0;
        loop {
            match field.chunk().await {
                Err(e) => {
                    return (
                        e.status(),
                        Html(html! {
                            <span class="text-red-500">{e}</span>
                        }),
                    )
                }
                Ok(Some(chunk)) => {
                    total += chunk.len();
                    if total as u64 > MAX_SIZE {
                        return (
                            StatusCode::PAYLOAD_TOO_LARGE,
                            Html(html! {
                                <span class="text-red-500">"File too large! Max size is " {ByteSize(MAX_SIZE)} "."</span>
                            }),
                        );
                    }
                    println!("received {} bytes (total {})", chunk.len(), total);
                }
                Ok(None) => break,
            }
        }
        println!("done reading `{}`, total {} bytes", name, total);
    }
    (
        StatusCode::OK,
        Html(html! {
            <span class="text-green-500">"Upload successful!"</span>
        }),
    )
}
