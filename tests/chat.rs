//! Integration tests for chat subcommand

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use assert_fs::prelude::*;

#[test]
fn no_args() {
    Command::cargo_bin("cogni")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn chat_no_message() {
    Command::cargo_bin("cogni")
        .unwrap()
        .args(["chat"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no messages provided"));
}

#[test]
fn chat_user_message_from_flag() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_header("content-type", "application/json")
        .with_header("authorization", "Bearer ABCDE")
        .match_body(mockito::Matcher::PartialJson(json!({
            "messages": [{
             "content": "Hello",
             "role": "user",
            }]
        })))
        .with_body(
            r#"{
             "id": "chatcmpl-XXXXX",
             "created": 1688413145,
             "model": "gpt-3.5-turbo-0613",
             "choices": [{
                 "index": 0,
                 "message": {
                     "role": "assistant",
                     "content": "ASSISTANT REPLY"
                 },
                 "finish_reason": "stop"
             }],
             "usage": {
                 "prompt_tokens": 8,
                 "completion_tokens": 9,
                 "total_tokens": 17
             }
        }"#,
        )
        .create();

    let cmd = Command::cargo_bin("cogni")
        .unwrap()
        .args(["chat", "-u", "Hello"])
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
        .mock("POST", "/v1/chat/completions")
        .with_header("content-type", "application/json")
        .with_header("authorization", "Bearer ABCDE")
        .match_body(mockito::Matcher::PartialJson(json!({
            "messages": [{
             "content": "Hello",
             "role": "user",
            }]
        })))
        .with_body(
            r#"{
             "id": "chatcmpl-XXXXX",
             "created": 1688413145,
             "model": "gpt-3.5-turbo-0613",
             "choices": [{
                 "index": 0,
                 "message": {
                     "role": "assistant",
                     "content": "ASSISTANT REPLY"
                 },
                 "finish_reason": "stop"
             }],
             "usage": {
                 "prompt_tokens": 8,
                 "completion_tokens": 9,
                 "total_tokens": 17
             }
        }"#,
        )
        .create();

    let cmd = Command::cargo_bin("cogni")
        .unwrap()
        .args(["chat"])
        .write_stdin("Hello")
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
        .mock("POST", "/v1/chat/completions")
        .with_header("content-type", "application/json")
        .with_header("authorization", "Bearer ABCDE")
        .match_body(mockito::Matcher::PartialJson(json!({
            "messages": [{
                "role": "system",
                "content": "SYSTEM",
            }, {
                "role": "user",
                "content": "USER_1",
            }, {
                "role": "assistant",
                "content": "ASSI_1",
            }, {
                "role": "user",
                "content": "USER_2",
            }, {
                "role": "assistant",
                "content": "ASSI_2",
            }, {
                "role": "user",
                "content": "USER_STDIN",
            }]
        })))
        .with_body(
            r#"{
             "id": "chatcmpl-XXXXX",
             "created": 1688413145,
             "model": "gpt-3.5-turbo-0613",
             "choices": [{
                 "index": 0,
                 "message": {
                     "role": "assistant",
                     "content": "ASSISTANT REPLY"
                 },
                 "finish_reason": "stop"
             }],
             "usage": {
                 "prompt_tokens": 8,
                 "completion_tokens": 9,
                 "total_tokens": 17
             }
        }"#,
        )
        .create();

    let cmd = Command::cargo_bin("cogni")
        .unwrap()
        .args([
            "chat", "-s", "SYSTEM", "-u", "USER_1", "-a", "ASSI_1", "-u", "USER_2", "-a", "ASSI_2",
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
        .mock("POST", "/v1/chat/completions")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_header("authorization", "Bearer ABCDE")
        .match_body(mockito::Matcher::PartialJson(json!({
            "messages": [{
                "role": "user",
                "content": "USER",
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
             }"# ,
        )
        .create();

    let cmd = Command::cargo_bin("cogni")
        .unwrap()
        .args([
            "chat", "-u", "USER", "-t", "1000"
        ])
        .write_stdin("USER_STDIN")
        .env("OPENAI_API_ENDPOINT", server.url())
        .env("OPENAI_API_KEY", "ABCDE")
        .assert();

    mock.assert();

    cmd.failure()
        .stderr(predicate::str::contains("1000 is greater than the maximum of 2"));
}

/// Test messages from file
#[test]
fn chat_user_message_from_file() {
    let mut server = mockito::Server::new();

    let infile = assert_fs::NamedTempFile::new("input.txt").unwrap();
    infile.write_str("Hello from file").unwrap();

    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_header("content-type", "application/json")
        .with_header("authorization", "Bearer ABCDE")
        .match_body(mockito::Matcher::PartialJson(json!({
            "messages": [{
                "role": "user",
                "content": "Hello from file",
            }],
        })))
        .with_body(
            r#"{
             "id": "chatcmpl-XXXXX",
             "created": 1688413145,
             "model": "gpt-3.5-turbo-0613",
             "choices": [{
                 "index": 0,
                 "message": {
                     "role": "assistant",
                     "content": "ASSISTANT REPLY"
                 },
                 "finish_reason": "stop"
             }],
             "usage": {
                 "prompt_tokens": 8,
                 "completion_tokens": 9,
                 "total_tokens": 17
             }
        }"#,
        )
        .create();

    let cmd = Command::cargo_bin("cogni")
        .unwrap()
        .args([
            "chat", infile.path().to_str().unwrap()
        ])
        .env("OPENAI_API_ENDPOINT", server.url())
        .env("OPENAI_API_KEY", "ABCDE")
        .assert();

    mock.assert();

    cmd.success()
        .stdout(predicate::str::contains("ASSISTANT REPLY"));
}

