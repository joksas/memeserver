use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{extract::DefaultBodyLimit, routing::post, Router};
use bytesize::{ByteSize, MB};
use html_to_string_macro::html;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .nest_service("/static", ServeDir::new("static"))
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
                <link rel="stylesheet" href="/static/dist/css/style.css" />
            </head>
            <body class="flex flex-col min-h-screen max-w-none">
                <main class="container mx-auto max-w-lg prose flex-grow">
                    <h1>{title}</h1>
                    <h2>"Upload file"</h2>
                    <form
                        class="grid grid-cols-1 gap-2 mx-auto"
                        hx-encoding="multipart/form-data"
                        hx-post="/upload"
                        _="on htmx:xhr:progress(loaded, total) set #progress.value to (loaded/total)*100"
                        >
                        <input type="file" name="file" required class="file:text-center file:px-5 file:py-2 file:no-underline file:font-bold file:border-0 hover:cursor-pointer hover:file:cursor-pointer file:outline-none file:w-1/2 file:text-blue-700 file:bg-blue-100 file:hover:bg-blue-200" />
                    </form>
                </main>
                <footer class="flex py-2">
                    <img src="https://htmx.org/img/createdwith.jpeg" alt="HTMX banner" class="inline-block h-20 w-auto mx-auto" width="680" height="168" />
                </footer>
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
