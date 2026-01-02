//! Concurrency tests for the Claude Agents SDK.
//!
//! These tests verify thread-safety and concurrent access patterns.

#![allow(clippy::type_complexity)]

use claude_agents_sdk::{
    AssistantMessage, BaseHookInput, ClaudeAgentOptions, ClaudeClientBuilder, ContentBlock,
    HookCallback, HookContext, HookEvent, HookInput, HookMatcher, HookOutput, Message,
    PermissionMode, PermissionResult, PreToolUseHookInput, ResultMessage, TextBlock,
    ToolPermissionContext,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Thread-Safety Tests
// ============================================================================

#[test]
fn test_options_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<ClaudeAgentOptions>();
}

#[test]
fn test_options_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<ClaudeAgentOptions>();
}

#[test]
fn test_message_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Message>();
}

#[test]
fn test_message_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<Message>();
}

#[test]
fn test_permission_result_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<PermissionResult>();
}

#[test]
fn test_permission_result_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<PermissionResult>();
}

#[test]
fn test_hook_output_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<HookOutput>();
}

#[test]
fn test_hook_output_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<HookOutput>();
}

// ============================================================================
// Concurrent Options Modification Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_options_building() {
    let handles: Vec<_> = (0..10)
        .map(|i| {
            tokio::spawn(async move {
                let options = ClaudeAgentOptions::new()
                    .with_model(format!("model-{}", i))
                    .with_max_turns(i as u32 + 1)
                    .with_system_prompt(format!("System prompt {}", i))
                    .with_timeout_secs(60 + i as u64);

                assert_eq!(options.model, Some(format!("model-{}", i)));
                assert_eq!(options.max_turns, Some(i as u32 + 1));
                assert_eq!(options.timeout_secs, Some(60 + i as u64));
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_arc_shared_options() {
    let options = Arc::new(
        ClaudeAgentOptions::new()
            .with_model("shared-model")
            .with_max_turns(5),
    );

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let opts = Arc::clone(&options);
            tokio::spawn(async move {
                assert_eq!(opts.model, Some("shared-model".to_string()));
                assert_eq!(opts.max_turns, Some(5));
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }
}

// ============================================================================
// Concurrent Callback Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_permission_callback_invocations() {
    let call_count = Arc::new(AtomicUsize::new(0));

    let callback_count = Arc::clone(&call_count);
    let callback: Arc<
        dyn Fn(
                String,
                serde_json::Value,
                ToolPermissionContext,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = PermissionResult> + Send>>
            + Send
            + Sync,
    > = Arc::new(move |_tool_name, _input, _ctx| {
        let count = Arc::clone(&callback_count);
        Box::pin(async move {
            count.fetch_add(1, Ordering::SeqCst);
            PermissionResult::allow()
        })
    });

    let handles: Vec<_> = (0..100)
        .map(|i| {
            let cb = Arc::clone(&callback);
            tokio::spawn(async move {
                let result = cb(
                    format!("Tool{}", i),
                    serde_json::json!({"arg": i}),
                    ToolPermissionContext::default(),
                )
                .await;
                assert!(matches!(result, PermissionResult::Allow(_)));
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(call_count.load(Ordering::SeqCst), 100);
}

#[tokio::test]
async fn test_concurrent_hook_callback_invocations() {
    let call_count = Arc::new(AtomicUsize::new(0));

    let callback_count = Arc::clone(&call_count);
    let callback: HookCallback = Arc::new(move |_input, _tool_use_id, _ctx| {
        let count = Arc::clone(&callback_count);
        Box::pin(async move {
            count.fetch_add(1, Ordering::SeqCst);
            HookOutput::default()
        })
    });

    let handles: Vec<_> = (0..100)
        .map(|_| {
            let cb = Arc::clone(&callback);
            tokio::spawn(async move {
                let input = HookInput::PreToolUse(PreToolUseHookInput {
                    base: BaseHookInput {
                        session_id: "test-session".to_string(),
                        transcript_path: "/tmp/test".to_string(),
                        cwd: "/".to_string(),
                        permission_mode: None,
                    },
                    hook_event_name: "PreToolUse".to_string(),
                    tool_name: "TestTool".to_string(),
                    tool_input: serde_json::json!({}),
                });
                let _output = cb(input, None, HookContext::default()).await;
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(call_count.load(Ordering::SeqCst), 100);
}

// ============================================================================
// Shared State Callback Tests
// ============================================================================

#[tokio::test]
async fn test_permission_callback_with_shared_state() {
    let allowed_tools = Arc::new(RwLock::new(vec!["Read".to_string(), "Write".to_string()]));

    let tools = Arc::clone(&allowed_tools);
    let callback: Arc<
        dyn Fn(
                String,
                serde_json::Value,
                ToolPermissionContext,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = PermissionResult> + Send>>
            + Send
            + Sync,
    > = Arc::new(move |tool_name, _input, _ctx| {
        let tools = Arc::clone(&tools);
        Box::pin(async move {
            let allowed = tools.read().await;
            if allowed.contains(&tool_name) {
                PermissionResult::allow()
            } else {
                PermissionResult::deny()
            }
        })
    });

    // Test concurrent reads
    let handles: Vec<_> = (0..50)
        .map(|i| {
            let cb = Arc::clone(&callback);
            let tool = if i % 2 == 0 { "Read" } else { "Bash" };
            tokio::spawn(async move {
                let result = cb(
                    tool.to_string(),
                    serde_json::json!({}),
                    ToolPermissionContext::default(),
                )
                .await;
                (tool, result)
            })
        })
        .collect();

    for handle in handles {
        let (tool, result) = handle.await.unwrap();
        if tool == "Read" {
            assert!(matches!(result, PermissionResult::Allow(_)));
        } else {
            assert!(matches!(result, PermissionResult::Deny(_)));
        }
    }

    // Test write while reading
    {
        let mut tools = allowed_tools.write().await;
        tools.push("Bash".to_string());
    }

    // Now Bash should be allowed
    let result = callback(
        "Bash".to_string(),
        serde_json::json!({}),
        ToolPermissionContext::default(),
    )
    .await;
    assert!(matches!(result, PermissionResult::Allow(_)));
}

#[tokio::test]
async fn test_hook_callback_with_shared_counter() {
    let tool_usage = Arc::new(RwLock::new(HashMap::<String, usize>::new()));

    let usage = Arc::clone(&tool_usage);
    let callback: HookCallback = Arc::new(move |input, _tool_use_id, _ctx| {
        let usage = Arc::clone(&usage);
        Box::pin(async move {
            if let HookInput::PreToolUse(pre) = input {
                let mut map = usage.write().await;
                *map.entry(pre.tool_name).or_insert(0) += 1;
            }
            HookOutput::default()
        })
    });

    let handles: Vec<_> = (0..100)
        .map(|i| {
            let cb = Arc::clone(&callback);
            let tool = match i % 3 {
                0 => "Read",
                1 => "Write",
                _ => "Bash",
            };
            tokio::spawn(async move {
                let input = HookInput::PreToolUse(PreToolUseHookInput {
                    base: BaseHookInput {
                        session_id: "test-session".to_string(),
                        transcript_path: "/tmp/test".to_string(),
                        cwd: "/".to_string(),
                        permission_mode: None,
                    },
                    hook_event_name: "PreToolUse".to_string(),
                    tool_name: tool.to_string(),
                    tool_input: serde_json::json!({}),
                });
                cb(input, None, HookContext::default()).await;
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }

    let usage = tool_usage.read().await;
    assert_eq!(*usage.get("Read").unwrap_or(&0), 34);
    assert_eq!(*usage.get("Write").unwrap_or(&0), 33);
    assert_eq!(*usage.get("Bash").unwrap_or(&0), 33);
}

// ============================================================================
// Concurrent Message Processing Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_message_creation() {
    let handles: Vec<_> = (0..100)
        .map(|i| {
            tokio::spawn(async move {
                let msg = Message::Assistant(AssistantMessage {
                    content: vec![ContentBlock::Text(TextBlock {
                        text: format!("Message {}", i),
                    })],
                    model: "claude-3".to_string(),
                    parent_tool_use_id: None,
                    error: None,
                });

                if let Message::Assistant(asst) = msg {
                    assert_eq!(asst.text(), format!("Message {}", i));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_shared_message_reading() {
    let messages: Vec<Arc<Message>> = (0..10)
        .map(|i| {
            Arc::new(Message::Assistant(AssistantMessage {
                content: vec![ContentBlock::Text(TextBlock {
                    text: format!("Message {}", i),
                })],
                model: "claude-3".to_string(),
                parent_tool_use_id: None,
                error: None,
            }))
        })
        .collect();

    let handles: Vec<_> = (0..100)
        .map(|i| {
            let msg = Arc::clone(&messages[i % 10]);
            tokio::spawn(async move {
                if let Message::Assistant(asst) = msg.as_ref() {
                    assert!(asst.text().starts_with("Message "));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }
}

// ============================================================================
// Hook Configuration Concurrent Access Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_hook_config_building() {
    let call_count = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let count = Arc::clone(&call_count);
            tokio::spawn(async move {
                let callback: HookCallback = Arc::new(move |_input, _tool_use_id, _ctx| {
                    let count = Arc::clone(&count);
                    Box::pin(async move {
                        count.fetch_add(1, Ordering::SeqCst);
                        HookOutput::default()
                    })
                });

                let mut hooks: HashMap<HookEvent, Vec<HookMatcher>> = HashMap::new();
                hooks.insert(
                    HookEvent::PreToolUse,
                    vec![HookMatcher {
                        matcher: Some(format!("Tool{}", i)),
                        hooks: vec![callback],
                        timeout: Some(5000.0),
                    }],
                );

                hooks
            })
        })
        .collect();

    let mut all_hooks = Vec::new();
    for handle in handles {
        all_hooks.push(handle.await.unwrap());
    }

    assert_eq!(all_hooks.len(), 10);
}

// ============================================================================
// Builder Pattern Concurrency Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_client_builder() {
    let handles: Vec<_> = (0..10)
        .map(|i| {
            tokio::spawn(async move {
                let callback_i = i;
                let _builder = ClaudeClientBuilder::new()
                    .model(format!("model-{}", i))
                    .max_turns(i as u32 + 1)
                    .permission_mode(PermissionMode::Default)
                    .can_use_tool(move |tool_name, _input, _ctx| {
                        let i = callback_i;
                        Box::pin(async move {
                            if tool_name.contains(&format!("{}", i)) {
                                PermissionResult::allow()
                            } else {
                                PermissionResult::deny()
                            }
                        })
                    });
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }
}

// ============================================================================
// Timeout Configuration Tests
// ============================================================================

#[test]
fn test_timeout_configuration() {
    let options = ClaudeAgentOptions::new().with_timeout_secs(120);

    assert_eq!(options.timeout_secs, Some(120));
}

#[test]
fn test_zero_timeout_disables() {
    let options = ClaudeAgentOptions::new().with_timeout_secs(0);

    assert_eq!(options.timeout_secs, Some(0));
}

#[test]
fn test_default_timeout() {
    let options = ClaudeAgentOptions::new();

    assert!(options.timeout_secs.is_none());
}

// ============================================================================
// Error Type Thread-Safety Tests
// ============================================================================

#[test]
fn test_error_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<claude_agents_sdk::ClaudeSDKError>();
}

#[test]
fn test_error_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<claude_agents_sdk::ClaudeSDKError>();
}

#[tokio::test]
async fn test_concurrent_error_creation() {
    let handles: Vec<_> = (0..100)
        .map(|i| {
            tokio::spawn(async move {
                let error = claude_agents_sdk::ClaudeSDKError::timeout(i as u64 * 100);
                assert!(error.to_string().contains("timed out"));
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }
}

// ============================================================================
// Result Type Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_result_message_access() {
    let result = Arc::new(ResultMessage {
        subtype: "success".to_string(),
        duration_ms: 1000,
        duration_api_ms: 800,
        is_error: false,
        num_turns: 5,
        session_id: "test-session".to_string(),
        total_cost_usd: Some(0.05),
        usage: None,
        result: Some("test result".to_string()),
        structured_output: None,
    });

    let handles: Vec<_> = (0..100)
        .map(|_| {
            let r = Arc::clone(&result);
            tokio::spawn(async move {
                assert_eq!(r.duration_ms, 1000);
                assert_eq!(r.num_turns, 5);
                assert_eq!(r.total_cost_usd, Some(0.05));
                assert!(!r.is_error);
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }
}

// ============================================================================
// Channel-Based Concurrent Communication Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_message_channel() {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(100);

    // Spawn producers
    let producer_handles: Vec<_> = (0..10)
        .map(|i| {
            let tx = tx.clone();
            tokio::spawn(async move {
                for j in 0..10 {
                    let msg = Message::Assistant(AssistantMessage {
                        content: vec![ContentBlock::Text(TextBlock {
                            text: format!("Producer {} Message {}", i, j),
                        })],
                        model: "claude-3".to_string(),
                        parent_tool_use_id: None,
                        error: None,
                    });
                    tx.send(msg).await.unwrap();
                }
            })
        })
        .collect();

    drop(tx); // Close the channel when all producers are done

    // Collect all messages
    let consumer_handle = tokio::spawn(async move {
        let mut count = 0;
        while let Some(msg) = rx.recv().await {
            if let Message::Assistant(asst) = msg {
                assert!(asst.text().contains("Producer"));
                count += 1;
            }
        }
        count
    });

    for handle in producer_handles {
        handle.await.unwrap();
    }

    let count = consumer_handle.await.unwrap();
    assert_eq!(count, 100);
}

// ============================================================================
// Stress Tests
// ============================================================================

#[tokio::test]
async fn test_high_concurrency_permission_callbacks() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let callback_count = Arc::clone(&call_count);

    let callback: Arc<
        dyn Fn(
                String,
                serde_json::Value,
                ToolPermissionContext,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = PermissionResult> + Send>>
            + Send
            + Sync,
    > = Arc::new(move |_tool_name, _input, _ctx| {
        let count = Arc::clone(&callback_count);
        Box::pin(async move {
            count.fetch_add(1, Ordering::SeqCst);
            // Simulate some async work
            tokio::task::yield_now().await;
            PermissionResult::allow()
        })
    });

    let handles: Vec<_> = (0..1000)
        .map(|i| {
            let cb = Arc::clone(&callback);
            tokio::spawn(async move {
                cb(
                    format!("Tool{}", i),
                    serde_json::json!({}),
                    ToolPermissionContext::default(),
                )
                .await
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(call_count.load(Ordering::SeqCst), 1000);
}

#[tokio::test]
async fn test_high_concurrency_message_creation() {
    let handles: Vec<_> = (0..1000)
        .map(|i| {
            tokio::spawn(async move {
                let msg = Message::Assistant(AssistantMessage {
                    content: vec![
                        ContentBlock::Text(TextBlock {
                            text: format!("Message {} part 1", i),
                        }),
                        ContentBlock::Text(TextBlock {
                            text: format!("Message {} part 2", i),
                        }),
                    ],
                    model: "claude-3".to_string(),
                    parent_tool_use_id: None,
                    error: None,
                });

                if let Message::Assistant(asst) = &msg {
                    assert_eq!(asst.content.len(), 2);
                }
                msg
            })
        })
        .collect();

    let mut messages = Vec::new();
    for handle in handles {
        messages.push(handle.await.unwrap());
    }

    assert_eq!(messages.len(), 1000);
}
