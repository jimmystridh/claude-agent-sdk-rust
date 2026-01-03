//! Conversation context and multi-turn tests.

#![cfg(feature = "integration-tests")]

use claude_agents_sdk::{ClaudeAgentOptions, ClaudeClient, PermissionMode};

/// Test that conversation context is maintained across turns.
///
/// Uses a simple math problem that requires remembering previous context.
#[tokio::test]
async fn test_conversation_context_maintained() {
    let options = ClaudeAgentOptions::new()
        .with_permission_mode(PermissionMode::Default)
        .with_max_turns(2);

    let mut client = ClaudeClient::new(Some(options), None);
    client.connect().await.expect("Failed to connect");

    // First turn: establish a number
    client
        .query("Let's do some math. Start with the number 15. Confirm by saying 'Starting with 15'.")
        .await
        .expect("Failed to send first query");

    let (response1, result1) = client
        .receive_response()
        .await
        .expect("First response failed");
    assert!(!result1.is_error);
    assert!(
        response1.contains("15"),
        "First response should acknowledge 15, got: {}",
        response1
    );

    // Second turn: use context to calculate
    client
        .query("Add 25 to that number. What's the total? Just give me the number.")
        .await
        .expect("Failed to send second query");

    let (response2, result2) = client
        .receive_response()
        .await
        .expect("Second response failed");

    assert!(!result2.is_error);
    assert!(
        response2.contains("40"),
        "Should compute 15+25=40, got: {}",
        response2
    );

    client.disconnect().await.expect("Failed to disconnect");
}

/// Test three-turn conversation maintains context.
#[tokio::test]
async fn test_three_turn_conversation() {
    let options = ClaudeAgentOptions::new()
        .with_permission_mode(PermissionMode::Default)
        .with_max_turns(3);

    let mut client = ClaudeClient::new(Some(options), None);
    client.connect().await.expect("Failed to connect");

    // Turn 1: Start with a number
    client
        .query("I'm thinking of the number 7. Say 'got it'.")
        .await
        .unwrap();
    let (_, r1) = client.receive_response().await.unwrap();
    assert!(!r1.is_error);

    // Turn 2: Add to it
    client
        .query("Add 3 to that number. What's the result?")
        .await
        .unwrap();
    let (resp2, r2) = client.receive_response().await.unwrap();
    assert!(!r2.is_error);
    assert!(resp2.contains("10"), "7+3 should be 10, got: {}", resp2);

    // Turn 3: Continue the chain
    client
        .query("Double that. What's the result?")
        .await
        .unwrap();
    let (resp3, r3) = client.receive_response().await.unwrap();
    assert!(!r3.is_error);
    assert!(resp3.contains("20"), "10*2 should be 20, got: {}", resp3);

    client.disconnect().await.expect("Failed to disconnect");
}
