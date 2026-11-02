//! The HTTP routes and the server's start/stop lifecycle.

use std::future::{self, Future};
use std::io;
use std::time::Duration;

use axum::extract::Path;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

#[derive(Serialize)]
struct Version {
    version: &'static str,
}

#[derive(Serialize)]
struct Greeting {
    message: String,
}

/// Builds the routes, with request logging already wired up.
pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/version", get(version))
        .route("/hello/{name}", get(hello))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
}

async fn healthz() -> Json<Health> {
    Json(Health { status: "ok" })
}

async fn version() -> Json<Version> {
    Json(Version {
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn hello(Path(name): Path<String>) -> Json<Greeting> {
    Json(Greeting {
        message: format!("Hello, {name}!"),
    })
}

/// Serves `app` on `listener` until `shutdown` completes, then stops accepting
/// connections and gives in-flight requests up to `timeout` to finish.
///
/// # Errors
///
/// Returns an error when the server fails, or a [`io::ErrorKind::TimedOut`]
/// error when requests are still running `timeout` after `shutdown`.
pub async fn serve(
    listener: TcpListener,
    app: Router,
    shutdown: impl Future<Output = ()> + Send + 'static,
    timeout: Duration,
) -> io::Result<()> {
    let (signalled, signal_received) = oneshot::channel::<()>();
    let server = async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown.await;
                // The receiver only goes away when the server has already stopped
                let _ = signalled.send(());
            })
            .await
    };
    tokio::pin!(server);

    // The timeout starts at the shutdown signal, not at startup
    let deadline = async {
        if signal_received.await.is_ok() {
            tokio::time::sleep(timeout).await;
        } else {
            future::pending::<()>().await;
        }
    };

    tokio::select! {
        result = &mut server => result,
        () = deadline => Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "in-flight requests did not finish before the shutdown timeout",
        )),
    }
}

/// Completes on Ctrl+C or, on Unix, SIGTERM.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        if tokio::signal::ctrl_c().await.is_err() {
            future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};

        match signal(SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(_) => future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    async fn get_body(path: &str) -> (StatusCode, String) {
        let request = Request::get(path).body(Body::empty()).unwrap();
        let response = router().oneshot(request).await.unwrap();
        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn healthz_reports_ok() {
        // Given the router
        // When /healthz is requested
        let (status, body) = get_body("/healthz").await;

        // Then it answers ok
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, r#"{"status":"ok"}"#);
    }

    #[tokio::test]
    async fn version_reports_the_crate_version() {
        // Given the router
        // When /version is requested
        let (status, body) = get_body("/version").await;

        // Then it answers with the version of the crate
        assert_eq!(status, StatusCode::OK);
        let expected = [r#"{"version":""#, env!("CARGO_PKG_VERSION"), r#""}"#].concat();
        assert_eq!(body, expected);
    }

    #[tokio::test]
    async fn hello_greets_the_name_in_the_path() {
        // Given the router
        // When /hello/Ferris is requested
        let (status, body) = get_body("/hello/Ferris").await;

        // Then it greets Ferris
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, r#"{"message":"Hello, Ferris!"}"#);
    }

    #[tokio::test]
    async fn unknown_paths_are_not_found() {
        // Given the router
        // When a path it does not have is requested
        let (status, _) = get_body("/nope").await;

        // Then it answers 404
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
