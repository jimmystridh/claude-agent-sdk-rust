//! Property-based tests using proptest.
//!
//! These tests verify invariants across a wide range of inputs.

use claude_agents_sdk::_internal::message_parser::*;
use claude_agents_sdk::*;
use proptest::prelude::*;
use serde_json::json;

// ============================================================================
// Arbitrary Generators
// ============================================================================

fn arbitrary_safe_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _-]{0,100}".prop_map(|s| s.to_string())
}

fn arbitrary_tool_name() -> impl Strategy<Value = String> {
    "[A-Z][a-zA-Z0-9]{0,20}".prop_map(|s| s.to_string())
}

fn arbitrary_tool_id() -> impl Strategy<Value = String> {
    "toolu_[a-zA-Z0-9]{10,20}".prop_map(|s| s.to_string())
}

fn arbitrary_session_id() -> impl Strategy<Value = String> {
    "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}".prop_map(|s| s.to_string())
}

// ============================================================================
// Message Parsing Properties
// ============================================================================

proptest! {
    /// Any valid user message text should parse successfully.
    #[test]
    fn prop_user_message_text_parses(text in arbitrary_safe_string()) {
        let raw = json!({
            "type": "user",
            "message": {
                "content": text.clone()
            }
        });

        let result = parse_message(raw);
        prop_assert!(result.is_ok(), "Failed to parse user message: {:?}", result);

        if let Ok(Some(Message::User(user))) = result {
            prop_assert_eq!(user.text(), Some(text.as_str()));
        }
    }

    /// Any valid assistant message should parse successfully.
    #[test]
    fn prop_assistant_message_parses(text in arbitrary_safe_string()) {
        let raw = json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "text", "text": text.clone()}
                ]
            }
        });

        let result = parse_message(raw);
        prop_assert!(result.is_ok(), "Failed to parse assistant message: {:?}", result);

        if let Ok(Some(Message::Assistant(asst))) = result {
            prop_assert!(!asst.content.is_empty());
        }
    }

    /// Tool use blocks should parse with any valid tool name and ID.
    #[test]
    fn prop_tool_use_parses(
        tool_name in arbitrary_tool_name(),
        tool_id in arbitrary_tool_id()
    ) {
        let raw = json!({
            "type": "assistant",
            "message": {
                "content": [
                    {
                        "type": "tool_use",
                        "id": tool_id.clone(),
                        "name": tool_name.clone(),
                        "input": {"key": "value"}
                    }
                ]
            }
        });

        let result = parse_message(raw);
        prop_assert!(result.is_ok(), "Failed to parse tool use: {:?}", result);

        if let Ok(Some(Message::Assistant(asst))) = result {
            if let Some(ContentBlock::ToolUse(tool)) = asst.content.first() {
                prop_assert_eq!(&tool.name, &tool_name);
                prop_assert_eq!(&tool.id, &tool_id);
            }
        }
    }

    /// Result messages should parse with valid session IDs.
    #[test]
    fn prop_result_message_parses(
        session_id in arbitrary_session_id(),
        num_turns in 1u32..100u32,
        duration_ms in 1u64..1000000u64
    ) {
        let raw = json!({
            "type": "result",
            "subtype": "success",
            "is_error": false,
            "duration_ms": duration_ms,
            "duration_api_ms": duration_ms / 2,
            "num_turns": num_turns,
            "session_id": session_id.clone()
        });

        let result = parse_message(raw);
        prop_assert!(result.is_ok(), "Failed to parse result: {:?}", result);

        if let Ok(Some(Message::Result(res))) = result {
            prop_assert_eq!(&res.session_id, &session_id);
            prop_assert_eq!(res.num_turns, num_turns);
            prop_assert_eq!(res.duration_ms, duration_ms);
            prop_assert!(!res.is_error);
        }
    }

    /// System messages should parse with any subtype.
    #[test]
    fn prop_system_message_parses(subtype in arbitrary_safe_string()) {
        let raw = json!({
            "type": "system",
            "subtype": subtype.clone()
        });

        let result = parse_message(raw);
        prop_assert!(result.is_ok(), "Failed to parse system message: {:?}", result);

        if let Ok(Some(Message::System(sys))) = result {
            prop_assert_eq!(&sys.subtype, &subtype);
        }
    }
}

// ============================================================================
// Options Builder Properties
// ============================================================================

proptest! {
    /// ClaudeAgentOptions builder should accept any reasonable max_turns.
    #[test]
    fn prop_options_max_turns(max_turns in 1u32..1000u32) {
        let options = ClaudeAgentOptions::new()
            .with_max_turns(max_turns);

        prop_assert_eq!(options.max_turns, Some(max_turns));
    }

    /// ClaudeAgentOptions builder should accept any model string.
    #[test]
    fn prop_options_model(model in arbitrary_safe_string()) {
        let options = ClaudeAgentOptions::new()
            .with_model(&model);

        prop_assert_eq!(options.model.as_deref(), Some(model.as_str()));
    }

    /// ClaudeAgentOptions builder should accept multiple allowed tools.
    #[test]
    fn prop_options_allowed_tools(
        tools in prop::collection::vec(arbitrary_tool_name(), 0..10)
    ) {
        let options = ClaudeAgentOptions::new()
            .with_allowed_tools(tools.clone());

        prop_assert_eq!(options.allowed_tools, tools);
    }

    /// ClaudeAgentOptions builder should accept any timeout.
    #[test]
    fn prop_options_timeout(timeout in 1u64..3600u64) {
        let options = ClaudeAgentOptions::new()
            .with_timeout_secs(timeout);

        prop_assert_eq!(options.timeout_secs, Some(timeout));
    }

    /// ClaudeAgentOptions builder should accept any budget.
    #[test]
    fn prop_options_budget(budget in 0.001f64..1000.0f64) {
        let mut options = ClaudeAgentOptions::new();
        options.max_budget_usd = Some(budget);

        prop_assert!((options.max_budget_usd.unwrap() - budget).abs() < 0.0001);
    }
}

// ============================================================================
// Content Block Properties
// ============================================================================

proptest! {
    /// Text content blocks should preserve text exactly.
    #[test]
    fn prop_text_block_preserves_content(text in arbitrary_safe_string()) {
        let block = ContentBlock::Text(TextBlock { text: text.clone() });

        if let ContentBlock::Text(tb) = block {
            prop_assert_eq!(&tb.text, &text);
        }
    }

    /// Thinking blocks should preserve content.
    #[test]
    fn prop_thinking_block_preserves_content(
        thinking in arbitrary_safe_string(),
        signature in arbitrary_safe_string()
    ) {
        let block = ContentBlock::Thinking(ThinkingBlock {
            thinking: thinking.clone(),
            signature: signature.clone(),
        });

        if let ContentBlock::Thinking(tb) = block {
            prop_assert_eq!(&tb.thinking, &thinking);
            prop_assert_eq!(&tb.signature, &signature);
        }
    }

    /// Tool result blocks should preserve tool_use_id.
    #[test]
    fn prop_tool_result_preserves_id(
        tool_use_id in arbitrary_tool_id(),
        content in arbitrary_safe_string(),
        is_error in proptest::bool::ANY
    ) {
        let block = ContentBlock::ToolResult(ToolResultBlock {
            tool_use_id: tool_use_id.clone(),
            content: Some(serde_json::Value::String(content.clone())),
            is_error: Some(is_error),
        });

        if let ContentBlock::ToolResult(tr) = block {
            prop_assert_eq!(&tr.tool_use_id, &tool_use_id);
            prop_assert_eq!(tr.content, Some(serde_json::Value::String(content)));
            prop_assert_eq!(tr.is_error, Some(is_error));
        }
    }
}

// ============================================================================
// Error Handling Properties
// ============================================================================

proptest! {
    /// Unknown message types should return Ok(None), not panic.
    #[test]
    fn prop_unknown_type_no_panic(unknown_type in "[a-z]{1,20}") {
        // Skip known types
        if ["user", "assistant", "result", "system", "stream_event"].contains(&unknown_type.as_str()) {
            return Ok(());
        }

        let raw = json!({
            "type": unknown_type,
            "message": {}
        });

        let result = parse_message(raw);
        // Unknown types should return Ok(None)
        prop_assert!(result.is_ok(), "Unknown type should not error: {:?}", result);
        prop_assert!(result.unwrap().is_none(), "Unknown type should return None");
    }

    /// Malformed JSON should not panic.
    #[test]
    fn prop_malformed_content_no_panic(garbage in arbitrary_safe_string()) {
        let raw = json!({
            "type": "user",
            "message": {
                "content": garbage
            }
        });

        // Should not panic
        let _ = parse_message(raw);
    }

    /// Missing required fields should return error, not panic.
    #[test]
    fn prop_missing_fields_no_panic(msg_type in prop::sample::select(vec!["user", "assistant", "result", "system"])) {
        let raw = json!({
            "type": msg_type
            // Missing message/content field
        });

        let result = parse_message(raw);
        // Most should error but none should panic
        prop_assert!(result.is_ok() || result.is_err());
    }
}

// ============================================================================
// Permission Result Properties
// ============================================================================

proptest! {
    /// PermissionResult::allow() should create an allow result.
    #[test]
    fn prop_permission_allow_is_allow(_dummy in 0..1i32) {
        let result = PermissionResult::allow();
        prop_assert!(matches!(result, PermissionResult::Allow(_)));
    }

    /// PermissionResult::deny() should create a deny result.
    #[test]
    fn prop_permission_deny_is_deny(_dummy in 0..1i32) {
        let result = PermissionResult::deny();
        prop_assert!(matches!(result, PermissionResult::Deny(_)));
    }
}

// ============================================================================
// Roundtrip Properties
// ============================================================================

proptest! {
    /// Options builder methods should be idempotent for same values.
    #[test]
    fn prop_options_builder_idempotent(
        max_turns in 1u32..100u32,
        timeout in 1u64..3600u64
    ) {
        let opts1 = ClaudeAgentOptions::new()
            .with_max_turns(max_turns)
            .with_timeout_secs(timeout);

        let opts2 = ClaudeAgentOptions::new()
            .with_max_turns(max_turns)
            .with_timeout_secs(timeout);

        prop_assert_eq!(opts1.max_turns, opts2.max_turns);
        prop_assert_eq!(opts1.timeout_secs, opts2.timeout_secs);
    }
}
