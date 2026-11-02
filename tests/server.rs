//! Integration tests: a real server on a free port, spoken to over TCP.

// clippy.toml only exempts #[test] functions, and these files' helpers are tests in spirit:
// a panic in them is the failure report.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use {{.Inputs.project_name.Value | replace "-" "_"}}::{router, serve};

/// Every wait in these tests is bounded, so a hang fails instead of stalling.
const WAIT: Duration = Duration::from_secs(5);

struct Running {
    addr: SocketAddr,
    stop: oneshot::Sender<()>,
    task: JoinHandle<std::io::Result<()>>,
}

async fn start() -> Running {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop, stopped) = oneshot::channel::<()>();
    let task = tokio::spawn(serve(
        listener,
        router(),
        async move {
            let _ = stopped.await;
        },
        Duration::from_secs(2),
    ));
    Running { addr, stop, task }
}

async fn get(addr: SocketAddr, path: &str) -> String {
    timeout(WAIT, async {
        let mut stream = TcpStream::connect(addr).await.unwrap();
        let request = format!("GET {path} HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).await.unwrap();
        response
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn serves_healthz_over_tcp() {
    // Given a running server
    let server = start().await;

    // When /healthz is requested
    let response = get(server.addr, "/healthz").await;

    // Then it answers 200 with the status
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.ends_with(r#"{"status":"ok"}"#), "{response}");

    // And it stops cleanly
    server.stop.send(()).unwrap();
    timeout(WAIT, server.task).await.unwrap().unwrap().unwrap();
}

#[tokio::test]
async fn stops_when_shutdown_is_signalled() {
    // Given a running server
    let server = start().await;

    // When shutdown is signalled
    server.stop.send(()).unwrap();

    // Then the server returns Ok and no longer accepts connections
    timeout(WAIT, server.task).await.unwrap().unwrap().unwrap();
    assert!(TcpStream::connect(server.addr).await.is_err());
}
