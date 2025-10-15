//! Integration tests for chat subcommand

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use serde_json::json;

#[test]
fn chat_no_message() {
    Command::cargo_bin("cogni")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("no messages provided"));
}

#[test]
fn chat_no_file() {
    Command::cargo_bin("cogni")
        .unwrap()
        .args(["file_does_not_exist"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "failed to open file_does_not_exist",
        ));
}

#[test]
fn chat_user_message_from_flag() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/v1/responses")
        .with_header("content-type", "application/json")
        .with_header("authorization", "Bearer ABCDE")
        .match_body(mockito::Matcher::PartialJson(json!({
            "model": "gpt-5",
            "input": [{
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": "Hello"
                }]
            }]
        })))
        .with_body(
            r#"{
             "id": "resp_XXXXX",
             "created": 1688413145,
             "model": "gpt-5",
             "output": [{
                 "id": "msg_XXXXX",
                 "type": "message",
                 "role": "assistant",
                 "content": [{
                     "type": "output_text",
                     "text": "ASSISTANT REPLY"
                 }]
             }],
             "usage": {
                 "input_tokens": 8,
                 "output_tokens": 9,
                 "total_tokens": 17
             }
        }"#,
        )
        .create();

    let cmd = Command::cargo_bin("cogni")
        .unwrap()
        .args(["-u", "Hello"])
        .env("OPENAI_API_ENDPOINT", server.url())
        .env("OPENAI_API_KEY", "ABCDE")
        .assert();

    mock.assert();

    cmd.success()
        .stdout(predicate::str::contains("ASSISTANT REPLY"));
}

#[test]
fn chat_user_message_from_stdin() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/v1/responses")
        .with_header("content-type", "application/json")
        .with_header("authorization", "Bearer ABCDE")
        .match_body(mockito::Matcher::PartialJson(json!({
            "model": "gpt-5",
            "input": [{
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": "Hello"
                }]
            }]
        })))
        .with_body(
            r#"{
             "id": "resp_XXXXX",
             "created": 1688413145,
             "model": "gpt-5",
             "output": [{
                 "id": "msg_XXXXX",
                 "type": "message",
                 "role": "assistant",
                 "content": [{
                     "type": "output_text",
                     "text": "ASSISTANT REPLY"
                 }]
             }],
            "usage": {
                "input_tokens": 8,
                "output_tokens": 9,
                "total_tokens": 17
            }
       }"#,
        )
        .create();

    let cmd = Command::cargo_bin("cogni")
        .unwrap()
        .write_stdin("Hello")
        .env("OPENAI_API_ENDPOINT", server.url())
        .env("OPENAI_API_KEY", "ABCDE")
        .assert();

    mock.assert();

    cmd.success()
        .stdout(predicate::str::contains("ASSISTANT REPLY"));
}

#[test]
fn chat_with_reasoning_effort() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/v1/responses")
        .with_header("content-type", "application/json")
        .with_header("authorization", "Bearer ABCDE")
        .match_body(mockito::Matcher::PartialJson(json!({
            "model": "gpt-5",
            "reasoning": {
                "effort": "medium"
            },
            "input": [{
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": "Hello"
                }]
            }]
        })))
        .with_body(
            r#"{
             "id": "resp_XXXXX",
             "created": 1688413145,
             "model": "gpt-5",
             "output": [{
                 "id": "msg_XXXXX",
                 "type": "message",
                 "role": "assistant",
                 "content": [{
                     "type": "output_text",
                     "text": "ASSISTANT REPLY"
                 }]
             }],
             "usage": {
                 "input_tokens": 8,
                 "output_tokens": 9,
                 "total_tokens": 17
             }
        }"#,
        )
        .create();

    let cmd = Command::cargo_bin("cogni")
        .unwrap()
        .args(["-u", "Hello", "--reasoning-effort", "medium"])
        .env("OPENAI_API_ENDPOINT", server.url())
        .env("OPENAI_API_KEY", "ABCDE")
        .assert();

    mock.assert();

    cmd.success()
        .stdout(predicate::str::contains("ASSISTANT REPLY"));
}

/// Test messages provided via
/// - System message flag
/// - Assistant message flag
/// - User message flag
/// - User message from stdin
#[test]
fn chat_multiple_messages() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/v1/responses")
        .with_header("content-type", "application/json")
        .with_header("authorization", "Bearer ABCDE")
        .match_body(mockito::Matcher::PartialJson(json!({
            "model": "gpt-5",
            "input": [{
                "role": "system",
                "content": [{
                    "type": "text",
                    "text": "SYSTEM"
                }],
            }, {
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": "USER_1"
                }],
            }, {
                "role": "assistant",
                "content": [{
                    "type": "text",
                    "text": "ASSI_1"
                }],
            }, {
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": "USER_2"
                }],
            }, {
                "role": "assistant",
                "content": [{
                    "type": "text",
                    "text": "ASSI_2"
                }],
            }, {
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": "USER_STDIN"
                }],
            }]
        })))
        .with_body(
            r#"{
             "id": "resp_XXXXX",
             "created": 1688413145,
             "model": "gpt-5",
             "output": [{
                 "id": "msg_XXXXX",
                 "type": "message",
                 "role": "assistant",
                 "content": [{
                     "type": "output_text",
                     "text": "ASSISTANT REPLY"
                 }]
             }],
             "usage": {
                 "input_tokens": 8,
                 "output_tokens": 9,
                 "total_tokens": 17
             }
        }"#,
        )
        .create();

    let cmd = Command::cargo_bin("cogni")
        .unwrap()
        .args([
            "-s", "SYSTEM", "-u", "USER_1", "-a", "ASSI_1", "-u", "USER_2", "-a", "ASSI_2",
        ])
        .write_stdin("USER_STDIN")
        .env("OPENAI_API_ENDPOINT", server.url())
        .env("OPENAI_API_KEY", "ABCDE")
        .assert();

    mock.assert();

    cmd.success()
        .stdout(predicate::str::contains("ASSISTANT REPLY"));
}

/// Test API errors are propagated
#[test]
fn chat_api_error() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/v1/responses")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_header("authorization", "Bearer ABCDE")
        .match_body(mockito::Matcher::PartialJson(json!({
            "model": "gpt-5",
            "input": [{
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": "USER"
                }],
            }],
            "temperature": 1000.0 // invalid param
        })))
        .with_body(
            r#"{
               "error": {
                 "message": "1000 is greater than the maximum of 2 - 'temperature'",
                 "type": "invalid_request_error",
                 "param": null,
                 "code": null
               }
             }"#,
        )
        .create();

    let cmd = Command::cargo_bin("cogni")
        .unwrap()
        .args(["-u", "USER", "-t", "1000"])
        .write_stdin("USER_STDIN")
        .env("OPENAI_API_ENDPOINT", server.url())
        .env("OPENAI_API_KEY", "ABCDE")
        .assert();

    mock.assert();

    cmd.failure().stderr(predicate::str::contains(
        "1000 is greater than the maximum of 2",
    ));
}

/// Test messages from file
#[test]
fn chat_user_message_from_file() {
    let mut server = mockito::Server::new();

    let infile = assert_fs::NamedTempFile::new("input.txt").unwrap();
    infile.write_str("Hello from file").unwrap();

    let mock = server
        .mock("POST", "/v1/responses")
        .with_header("content-type", "application/json")
        .with_header("authorization", "Bearer ABCDE")
        .match_body(mockito::Matcher::PartialJson(json!({
            "model": "gpt-5",
            "input": [{
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": "Hello from file"
                }],
            }],
        })))
        .with_body(
            r#"{
             "id": "resp_XXXXX",
             "created": 1688413145,
             "model": "gpt-5",
             "output": [{
                 "id": "msg_XXXXX",
                 "type": "message",
                 "role": "assistant",
                 "content": [{
                     "type": "output_text",
                     "text": "ASSISTANT REPLY"
                 }]
             }],
             "usage": {
                 "input_tokens": 8,
                 "output_tokens": 9,
                 "total_tokens": 17
             }
        }"#,
        )
        .create();

    let cmd = Command::cargo_bin("cogni")
        .unwrap()
        .args([infile.path().to_str().unwrap()])
        .env("OPENAI_API_ENDPOINT", server.url())
        .env("OPENAI_API_KEY", "ABCDE")
        .assert();

    mock.assert();

    cmd.success()
        .stdout(predicate::str::contains("ASSISTANT REPLY"));
}
