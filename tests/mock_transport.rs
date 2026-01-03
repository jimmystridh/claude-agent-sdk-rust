//! Mock transport for deterministic testing.
//!
//! This module provides a mock transport that returns pre-recorded responses,
//! allowing for deterministic unit testing without requiring the actual CLI.

use async_trait::async_trait;
use claude_agents_sdk::Result;
use claude_agents_sdk::_internal::transport::Transport;
use futures::stream;
use serde_json::{json, Value};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio_stream::Stream;

/// A mock transport that returns pre-recorded responses.
pub struct MockTransport {
    /// Pre-recorded responses to return (in order).
    responses: Arc<Mutex<Vec<Value>>>,
    /// Index of the next response to return.
    response_index: Arc<AtomicUsize>,
    /// Whether the transport is connected.
    connected: AtomicBool,
    /// Messages written to the transport.
    written_messages: Arc<Mutex<Vec<String>>>,
}

impl MockTransport {
    /// Create a new mock transport with the given responses.
    pub fn new(responses: Vec<Value>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            response_index: Arc::new(AtomicUsize::new(0)),
            connected: AtomicBool::new(false),
            written_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a mock transport with a simple text response.
    pub fn with_text_response(text: &str) -> Self {
        Self::new(vec![
            json!({
                "type": "system",
                "subtype": "init",
                "data": {"session_id": "mock-session"}
            }),
            json!({
                "type": "assistant",
                "message": {
                    "content": [{"type": "text", "text": text}],
                    "model": "mock-model"
                }
            }),
            json!({
                "type": "result",
                "subtype": "success",
                "is_error": false,
                "duration_ms": 100,
                "duration_api_ms": 80,
                "num_turns": 1,
                "session_id": "mock-session",
                "total_cost_usd": 0.001
            }),
        ])
    }

    /// Create a mock transport that simulates an error.
    pub fn with_error_response(error_message: &str) -> Self {
        Self::new(vec![
            json!({
                "type": "system",
                "subtype": "init",
                "data": {"session_id": "mock-session"}
            }),
            json!({
                "type": "result",
                "subtype": "error",
                "is_error": true,
                "duration_ms": 50,
                "duration_api_ms": 40,
                "num_turns": 0,
                "session_id": "mock-session",
                "result": error_message
            }),
        ])
    }

    /// Create a mock transport that simulates tool use.
    pub fn with_tool_use(tool_name: &str, tool_input: Value) -> Self {
        Self::new(vec![
            json!({
                "type": "system",
                "subtype": "init",
                "data": {"session_id": "mock-session"}
            }),
            json!({
                "type": "assistant",
                "message": {
                    "content": [
                        {"type": "text", "text": "Let me use a tool."},
                        {
                            "type": "tool_use",
                            "id": "mock-tool-id",
                            "name": tool_name,
                            "input": tool_input
                        }
                    ],
                    "model": "mock-model"
                }
            }),
            json!({
                "type": "result",
                "subtype": "success",
                "is_error": false,
                "duration_ms": 200,
                "duration_api_ms": 150,
                "num_turns": 1,
                "session_id": "mock-session"
            }),
        ])
    }

    /// Get messages that were written to the transport.
    pub fn get_written_messages(&self) -> Vec<String> {
        self.written_messages.lock().unwrap().clone()
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn connect(&mut self) -> Result<()> {
        self.connected.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn write(&self, data: &str) -> Result<()> {
        self.written_messages.lock().unwrap().push(data.to_string());
        Ok(())
    }

    fn message_stream(&self) -> Pin<Box<dyn Stream<Item = Result<Value>> + Send + '_>> {
        let responses = self.responses.clone();
        let index = self.response_index.clone();

        Box::pin(stream::iter(std::iter::from_fn(move || {
            let idx = index.fetch_add(1, Ordering::SeqCst);
            let responses = responses.lock().unwrap();
            if idx < responses.len() {
                Some(Ok(responses[idx].clone()))
            } else {
                None
            }
        })))
    }

    async fn close(&mut self) -> Result<()> {
        self.connected.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn end_input(&self) -> Result<()> {
        Ok(())
    }

    fn is_ready(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }
}

// ============================================================================
// Mock Transport Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_mock_transport_connect_and_close() {
        let mut transport = MockTransport::new(vec![]);

        assert!(!transport.is_ready(), "Should not be ready before connect");

        transport.connect().await.unwrap();
        assert!(transport.is_ready(), "Should be ready after connect");

        transport.close().await.unwrap();
        assert!(!transport.is_ready(), "Should not be ready after close");
    }

    #[tokio::test]
    async fn test_mock_transport_write_captures_messages() {
        let transport = MockTransport::new(vec![]);

        transport.write("Hello").await.unwrap();
        transport.write("World").await.unwrap();

        let messages = transport.get_written_messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], "Hello");
        assert_eq!(messages[1], "World");
    }

    #[tokio::test]
    async fn test_mock_transport_returns_responses_in_order() {
        let transport = MockTransport::new(vec![
            json!({"index": 0}),
            json!({"index": 1}),
            json!({"index": 2}),
        ]);

        let mut stream = transport.message_stream();

        let msg0 = stream.next().await.unwrap().unwrap();
        assert_eq!(msg0["index"], 0);

        let msg1 = stream.next().await.unwrap().unwrap();
        assert_eq!(msg1["index"], 1);

        let msg2 = stream.next().await.unwrap().unwrap();
        assert_eq!(msg2["index"], 2);

        // Should return None when exhausted
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_mock_transport_with_text_response() {
        let transport = MockTransport::with_text_response("Hello, world!");
        let mut stream = transport.message_stream();

        // System init
        let init = stream.next().await.unwrap().unwrap();
        assert_eq!(init["type"], "system");

        // Assistant response
        let response = stream.next().await.unwrap().unwrap();
        assert_eq!(response["type"], "assistant");
        assert_eq!(response["message"]["content"][0]["text"], "Hello, world!");

        // Result
        let result = stream.next().await.unwrap().unwrap();
        assert_eq!(result["type"], "result");
        assert_eq!(result["is_error"], false);
    }

    #[tokio::test]
    async fn test_mock_transport_with_error_response() {
        let transport = MockTransport::with_error_response("Something went wrong");
        let mut stream = transport.message_stream();

        // System init
        let _init = stream.next().await.unwrap().unwrap();

        // Error result
        let result = stream.next().await.unwrap().unwrap();
        assert_eq!(result["type"], "result");
        assert_eq!(result["is_error"], true);
        assert_eq!(result["result"], "Something went wrong");
    }

    #[tokio::test]
    async fn test_mock_transport_with_tool_use() {
        let transport = MockTransport::with_tool_use("Bash", json!({"command": "ls"}));
        let mut stream = transport.message_stream();

        // System init
        let _init = stream.next().await.unwrap().unwrap();

        // Assistant with tool use
        let response = stream.next().await.unwrap().unwrap();
        assert_eq!(response["type"], "assistant");

        let content = &response["message"]["content"];
        assert_eq!(content[1]["type"], "tool_use");
        assert_eq!(content[1]["name"], "Bash");
        assert_eq!(content[1]["input"]["command"], "ls");
    }
}
