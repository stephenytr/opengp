use opengp_domain::domain::api::{ApiErrorResponse, LoginResponse};
use serde_json::json;
use std::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

fn reserve_test_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("should reserve local test port");
    listener
        .local_addr()
        .expect("reserved listener should have local address")
        .port()
}

async fn send_json_request(
    port: u16,
    method: &str,
    path: &str,
    body: Option<String>,
    bearer_token: Option<&str>,
) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("should connect to API server");

    let body = body.unwrap_or_default();
    let auth_header = bearer_token
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();

    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\nContent-Type: application/json\r\n{auth_header}Content-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );

    stream
        .write_all(request.as_bytes())
        .await
        .expect("should write HTTP request");

    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .await
        .expect("should read HTTP response");

    let response = String::from_utf8(raw).expect("response should be UTF-8");
    let (headers, response_body) = response
        .split_once("\r\n\r\n")
        .expect("response should contain headers and body");

    let status = headers
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .expect("status line should contain valid HTTP status code");

    (status, response_body.to_string())
}

async fn wait_for_server(port: u16) {
    for _ in 0..50 {
        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("API server did not start in time on port {port}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_booking_returns_conflict_for_second_writer() {
    let port = reserve_test_port();

    unsafe {
        std::env::set_var("API_HOST", "127.0.0.1");
        std::env::set_var("API_PORT", port.to_string());
        std::env::set_var(
            "API_DATABASE_URL",
            "postgres://postgres:postgres@127.0.0.1:5432/opengp",
        );
        std::env::set_var(
            "ENCRYPTION_KEY",
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
        std::env::set_var("LOG_LEVEL", "warn");
    }

    let server_handle = tokio::spawn(async { opengp_api::run().await });
    wait_for_server(port).await;

    let login_body = json!({
        "username": "dr_smith",
        "password": "correct-horse-battery-staple"
    })
    .to_string();

    let (login_status, login_response_body) = send_json_request(
        port,
        "POST",
        "/api/v1/auth/login",
        Some(login_body),
        None,
    )
    .await;
    assert_eq!(login_status, 200, "login must succeed before booking");

    let login: LoginResponse =
        serde_json::from_str(&login_response_body).expect("login response should be valid JSON");

    let practitioner_id = Uuid::new_v4();
    let patient_id = Uuid::new_v4();
    let start_time = (chrono::Utc::now() + chrono::Duration::hours(2)).to_rfc3339();

    let appointment_payload = json!({
        "patient_id": patient_id,
        "practitioner_id": practitioner_id,
        "start_time": start_time,
        "duration_minutes": 15,
        "appointment_type": "standard",
        "reason": "Concurrent booking test",
        "is_urgent": false
    })
    .to_string();

    let first_token = login.access_token.clone();
    let first_payload = appointment_payload.clone();
    let first = tokio::spawn(async move {
        send_json_request(
            port,
            "POST",
            "/api/v1/appointments",
            Some(first_payload),
            Some(&first_token),
        )
        .await
    });

    let second_token = login.access_token.clone();
    let second_payload = appointment_payload;
    let second = tokio::spawn(async move {
        sleep(Duration::from_millis(20)).await;
        send_json_request(
            port,
            "POST",
            "/api/v1/appointments",
            Some(second_payload),
            Some(&second_token),
        )
        .await
    });

    let (first_status, _first_body) = first.await.expect("first task should complete");
    let (second_status, second_body) = second.await.expect("second task should complete");

    assert_eq!(first_status, 201, "first booking should succeed");
    assert_eq!(second_status, 409, "second booking should return conflict");

    let conflict: ApiErrorResponse =
        serde_json::from_str(&second_body).expect("conflict response should be valid JSON");
    assert_eq!(conflict.status, 409);
    assert_eq!(conflict.code, "appointment_conflict");
    assert!(
        conflict.message.to_lowercase().contains("overlapping"),
        "conflict message should clearly describe overlap, got: {}",
        conflict.message
    );

    server_handle.abort();
}
