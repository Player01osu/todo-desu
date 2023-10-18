use axum::response::Redirect;
use axum::{
    extract::Query,
    response::IntoResponse,
    routing::{get, get_service, post},
    Router,
};
use shrimple::Shrimpipe;
use std::process::Command;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tower_http::services::ServeFile;

pub const TODO: &'static str = "/home/bruh/todo.wiki";

fn sanitize(s: impl AsRef<str>) -> String {
    fn sanitize_(s: &str) -> String {
        s.chars().filter(|c| *c != '\r').collect::<String>()
    }
    sanitize_(s.as_ref())
}

pub async fn write_file(blob: &str) {
        let blob = sanitize(blob);
        let mut patch = Command::new("patch");
        let patch = patch.args([TODO]);
        let mut diff = Command::new("diff")
            .args([TODO, "-"])
            .stdin_write(blob.as_bytes())
            .unwrap()
            .pipe(patch)
            .unwrap();
        diff.wait().unwrap();
        let mut f = File::create("./assets/index.html").await.unwrap();
        f.write_all(
            format!(
                r#"
<!DOCTYPE html>
<html lang="en">
    <head>
        <title>Todo</title>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <link href="css/style.css" rel="stylesheet">
    </head>
    <body>
        <form action="/api/v1/new-todo" method="get" style="display: flex">
            <label for="todo"></label>
            <textarea id="todo" name="todo" rows="300" cols="100">
{}
            </textarea>
            <input type="submit" value="Submit">
        </form>
    </body>
</html>"#,
                blob.as_str()
            )
            .as_bytes(),
        )
        .await
        .unwrap();
}

pub async fn put_file(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    if let Some(blob) = params.get("todo") {
        write_file(blob).await;
    }
    Redirect::permanent("/")
}

pub async fn post_file(body: String) -> impl IntoResponse {
    write_file(&body).await;
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 42069);

    let mime = "text/plain".parse::<mime::Mime>().unwrap();
    let api = Router::new()
        .route(
            "/",
            get_service(ServeFile::new_with_mime(TODO, &mime)),
        )
        .route("/", post(post_file))
        .route("/new-todo", get(put_file));

    let app = Router::new()
        .route("/", get_service(ServeFile::new("./assets/index.html")))
        .nest("/api/v1", api);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
