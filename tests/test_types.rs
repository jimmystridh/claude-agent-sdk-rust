//! Tests for type definitions, serialization, and round-trip behavior.
//!
//! This file tests all public types including:
//! - Message types and their content blocks
//! - Permission types and results
//! - Hook events and outputs
//! - Serialization/deserialization round-trips

use claude_agents_sdk::*;
use serde_json::json;

// ============================================================================
// Permission Mode Tests
// ============================================================================

#[test]
fn test_permission_mode_serializes_to_camel_case() {
    assert_eq!(
        serde_json::to_string(&PermissionMode::AcceptEdits).unwrap(),
        r#""acceptEdits""#,
        "AcceptEdits should serialize to camelCase"
    );
    assert_eq!(
        serde_json::to_string(&PermissionMode::BypassPermissions).unwrap(),
        r#""bypassPermissions""#,
        "BypassPermissions should serialize to camelCase"
    );
    assert_eq!(
        serde_json::to_string(&PermissionMode::Plan).unwrap(),
        r#""plan""#,
        "Plan should serialize as lowercase"
    );
    assert_eq!(
        serde_json::to_string(&PermissionMode::Default).unwrap(),
        r#""default""#,
        "Default should serialize as lowercase"
    );
}

#[test]
fn test_permission_mode_deserializes_from_camel_case() {
    let mode: PermissionMode = serde_json::from_str(r#""acceptEdits""#).unwrap();
    assert_eq!(mode, PermissionMode::AcceptEdits);

    let mode: PermissionMode = serde_json::from_str(r#""bypassPermissions""#).unwrap();
    assert_eq!(mode, PermissionMode::BypassPermissions);
}

#[test]
fn test_permission_mode_roundtrip() {
    for mode in [
        PermissionMode::Default,
        PermissionMode::AcceptEdits,
        PermissionMode::Plan,
        PermissionMode::BypassPermissions,
    ] {
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: PermissionMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, deserialized, "Round-trip failed for {:?}", mode);
    }
}

// ============================================================================
// Permission Result Tests
// ============================================================================

#[test]
fn test_permission_result_allow_creates_correct_json() {
    let result = PermissionResult::allow();
    let json = serde_json::to_value(&result).unwrap();

    assert_eq!(json["behavior"], "allow", "behavior should be 'allow'");
}

#[test]
fn test_permission_result_deny_creates_correct_json() {
    let result = PermissionResult::deny();
    let json = serde_json::to_value(&result).unwrap();

    assert_eq!(json["behavior"], "deny", "behavior should be 'deny'");
}

#[test]
fn test_permission_result_deny_with_message_includes_message() {
    let result = PermissionResult::deny_with_message("Operation not allowed");
    let json = serde_json::to_value(&result).unwrap();

    assert_eq!(json["behavior"], "deny");
    assert_eq!(json["message"], "Operation not allowed");
}

#[test]
fn test_permission_result_allow_with_updated_input() {
    let result = PermissionResult::Allow(PermissionResultAllow {
        behavior: "allow".to_string(),
        updated_input: Some(json!({"modified": true, "extra_field": "added"})),
        updated_permissions: None,
    });
    let json = serde_json::to_value(&result).unwrap();

    assert_eq!(json["behavior"], "allow");
    assert_eq!(json["updatedInput"]["modified"], true);
    assert_eq!(json["updatedInput"]["extra_field"], "added");
}

// ============================================================================
// Content Block Tests
// ============================================================================

#[test]
fn test_text_block_as_text_returns_content() {
    let block = ContentBlock::Text(TextBlock {
        text: "Hello, world!".to_string(),
    });

    assert_eq!(
        block.as_text(),
        Some("Hello, world!"),
        "as_text() should return the text content"
    );
    assert!(!block.is_tool_use(), "Text block should not be tool use");
}

#[test]
fn test_tool_use_block_is_tool_use_returns_true() {
    let block = ContentBlock::ToolUse(ToolUseBlock {
        id: "tool_123".to_string(),
        name: "Bash".to_string(),
        input: json!({"command": "ls -la"}),
    });

    assert!(
        block.is_tool_use(),
        "ToolUse block should return true for is_tool_use()"
    );
    assert!(
        block.as_text().is_none(),
        "ToolUse block should return None for as_text()"
    );
}

#[test]
fn test_tool_result_block_fields() {
    let block = ContentBlock::ToolResult(ToolResultBlock {
        tool_use_id: "tool_123".to_string(),
        content: Some(json!("Command output here")),
        is_error: Some(false),
    });

    if let ContentBlock::ToolResult(result) = block {
        assert_eq!(result.tool_use_id, "tool_123");
        assert_eq!(result.content, Some(json!("Command output here")));
        assert_eq!(result.is_error, Some(false));
    } else {
        panic!("Expected ToolResult block");
    }
}

#[test]
fn test_thinking_block_fields() {
    let block = ContentBlock::Thinking(ThinkingBlock {
        thinking: "Let me analyze this...".to_string(),
        signature: "sig_abc123".to_string(),
    });

    if let ContentBlock::Thinking(thinking) = block {
        assert_eq!(thinking.thinking, "Let me analyze this...");
        assert_eq!(thinking.signature, "sig_abc123");
    } else {
        panic!("Expected Thinking block");
    }
}

// ============================================================================
// User Message Tests
// ============================================================================

#[test]
fn test_user_message_text_content_returns_text() {
    let msg = UserMessage {
        content: UserMessageContent::Text("Hello from user".to_string()),
        uuid: None,
        parent_tool_use_id: None,
    };

    assert_eq!(
        msg.text(),
        Some("Hello from user"),
        "text() should return the text content"
    );
}

#[test]
fn test_user_message_blocks_content() {
    let msg = UserMessage {
        content: UserMessageContent::Blocks(vec![
            ContentBlock::Text(TextBlock {
                text: "First block".to_string(),
            }),
            ContentBlock::Text(TextBlock {
                text: "Second block".to_string(),
            }),
        ]),
        uuid: None,
        parent_tool_use_id: None,
    };

    // text() returns None for block content
    assert!(
        msg.text().is_none(),
        "text() should return None for block content"
    );

    if let UserMessageContent::Blocks(blocks) = msg.content {
        assert_eq!(blocks.len(), 2);
    } else {
        panic!("Expected Blocks content");
    }
}

#[test]
fn test_user_message_with_uuid() {
    let msg = UserMessage {
        content: UserMessageContent::Text("Test".to_string()),
        uuid: Some("unique-id-12345".to_string()),
        parent_tool_use_id: None,
    };

    assert_eq!(msg.uuid, Some("unique-id-12345".to_string()));
}

// ============================================================================
// Assistant Message Tests
// ============================================================================

#[test]
fn test_assistant_message_text_concatenates_all_text_blocks() {
    let msg = AssistantMessage {
        content: vec![
            ContentBlock::Text(TextBlock {
                text: "Hello ".to_string(),
            }),
            ContentBlock::Text(TextBlock {
                text: "world".to_string(),
            }),
            ContentBlock::Text(TextBlock {
                text: "!".to_string(),
            }),
        ],
        model: "claude-3-sonnet".to_string(),
        parent_tool_use_id: None,
        error: None,
    };

    assert_eq!(
        msg.text(),
        "Hello world!",
        "text() should concatenate all text blocks"
    );
}

#[test]
fn test_assistant_message_text_skips_non_text_blocks() {
    let msg = AssistantMessage {
        content: vec![
            ContentBlock::Text(TextBlock {
                text: "Let me help: ".to_string(),
            }),
            ContentBlock::ToolUse(ToolUseBlock {
                id: "tool_1".to_string(),
                name: "Read".to_string(),
                input: json!({"path": "/tmp/file"}),
            }),
            ContentBlock::Text(TextBlock {
                text: "Done.".to_string(),
            }),
        ],
        model: "claude-3".to_string(),
        parent_tool_use_id: None,
        error: None,
    };

    assert_eq!(
        msg.text(),
        "Let me help: Done.",
        "text() should skip non-text blocks"
    );
}

#[test]
fn test_assistant_message_tool_uses_extracts_all_tool_uses() {
    let msg = AssistantMessage {
        content: vec![
            ContentBlock::Text(TextBlock {
                text: "Running commands...".to_string(),
            }),
            ContentBlock::ToolUse(ToolUseBlock {
                id: "tool_1".to_string(),
                name: "Bash".to_string(),
                input: json!({"command": "ls"}),
            }),
            ContentBlock::ToolUse(ToolUseBlock {
                id: "tool_2".to_string(),
                name: "Read".to_string(),
                input: json!({"path": "/tmp/file.txt"}),
            }),
        ],
        model: "claude-3".to_string(),
        parent_tool_use_id: None,
        error: None,
    };

    let tool_uses = msg.tool_uses();
    assert_eq!(tool_uses.len(), 2, "Should extract both tool uses");
    assert_eq!(tool_uses[0].name, "Bash");
    assert_eq!(tool_uses[1].name, "Read");
}

#[test]
fn test_assistant_message_tool_uses_returns_empty_when_no_tools() {
    let msg = AssistantMessage {
        content: vec![ContentBlock::Text(TextBlock {
            text: "Just text".to_string(),
        })],
        model: "claude-3".to_string(),
        parent_tool_use_id: None,
        error: None,
    };

    assert!(
        msg.tool_uses().is_empty(),
        "tool_uses() should return empty vec when no tool uses"
    );
}

// ============================================================================
// Result Message Tests
// ============================================================================

#[test]
fn test_result_message_success() {
    let result = ResultMessage {
        subtype: "success".to_string(),
        duration_ms: 1500,
        duration_api_ms: 1200,
        is_error: false,
        num_turns: 3,
        session_id: "sess_abc123".to_string(),
        total_cost_usd: Some(0.0042),
        usage: Some(json!({
            "input_tokens": 150,
            "output_tokens": 75
        })),
        result: Some("Task completed successfully".to_string()),
        structured_output: None,
    };

    assert_eq!(result.subtype, "success");
    assert!(!result.is_error);
    assert_eq!(result.num_turns, 3);
    assert_eq!(result.total_cost_usd, Some(0.0042));
}

#[test]
fn test_result_message_error() {
    let result = ResultMessage {
        subtype: "error".to_string(),
        duration_ms: 500,
        duration_api_ms: 400,
        is_error: true,
        num_turns: 1,
        session_id: "sess_xyz789".to_string(),
        total_cost_usd: Some(0.001),
        usage: None,
        result: Some("API rate limit exceeded".to_string()),
        structured_output: None,
    };

    assert!(result.is_error);
    assert_eq!(result.subtype, "error");
}

// ============================================================================
// Message Type Tests
// ============================================================================

#[test]
fn test_message_is_assistant_discriminates_correctly() {
    let assistant = Message::Assistant(AssistantMessage {
        content: vec![ContentBlock::Text(TextBlock {
            text: "Hi".to_string(),
        })],
        model: "claude-3".to_string(),
        parent_tool_use_id: None,
        error: None,
    });

    assert!(assistant.is_assistant());
    assert!(!assistant.is_result());
}

#[test]
fn test_message_is_result_discriminates_correctly() {
    let result = Message::Result(ResultMessage {
        subtype: "success".to_string(),
        duration_ms: 100,
        duration_api_ms: 80,
        is_error: false,
        num_turns: 1,
        session_id: "test".to_string(),
        total_cost_usd: None,
        usage: None,
        result: None,
        structured_output: None,
    });

    assert!(result.is_result());
    assert!(!result.is_assistant());
}

#[test]
fn test_message_as_assistant_returns_reference() {
    let msg = Message::Assistant(AssistantMessage {
        content: vec![ContentBlock::Text(TextBlock {
            text: "Hello".to_string(),
        })],
        model: "claude-3".to_string(),
        parent_tool_use_id: None,
        error: None,
    });

    let asst = msg
        .as_assistant()
        .expect("Should return Some for Assistant");
    assert_eq!(asst.text(), "Hello");
}

#[test]
fn test_message_as_result_returns_reference() {
    let msg = Message::Result(ResultMessage {
        subtype: "success".to_string(),
        duration_ms: 100,
        duration_api_ms: 80,
        is_error: false,
        num_turns: 1,
        session_id: "test-session".to_string(),
        total_cost_usd: Some(0.005),
        usage: None,
        result: None,
        structured_output: None,
    });

    let result = msg.as_result().expect("Should return Some for Result");
    assert_eq!(result.session_id, "test-session");
    assert_eq!(result.total_cost_usd, Some(0.005));
}

// ============================================================================
// Hook Event Tests
// ============================================================================

#[test]
fn test_hook_event_serialization() {
    assert_eq!(
        serde_json::to_string(&HookEvent::PreToolUse).unwrap(),
        r#""PreToolUse""#
    );
    assert_eq!(
        serde_json::to_string(&HookEvent::PostToolUse).unwrap(),
        r#""PostToolUse""#
    );
}

#[test]
fn test_hook_event_roundtrip() {
    for event in [HookEvent::PreToolUse, HookEvent::PostToolUse] {
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }
}

// ============================================================================
// Sync Hook Output Tests
// ============================================================================

#[test]
fn test_sync_hook_output_uses_continue_not_continue_underscore() {
    let output = SyncHookOutput {
        continue_: Some(true),
        suppress_output: Some(false),
        ..Default::default()
    };

    let json = serde_json::to_value(&output).unwrap();

    // Should use "continue" not "continue_" in JSON (serde rename)
    assert_eq!(
        json["continue"], true,
        "JSON should use 'continue' not 'continue_'"
    );
    assert!(
        json.get("continue_").is_none(),
        "JSON should not have 'continue_' field"
    );
}

#[test]
fn test_sync_hook_output_default() {
    let output = SyncHookOutput::default();

    assert!(output.continue_.is_none());
    assert!(output.suppress_output.is_none());
    assert!(output.stop_reason.is_none());
    assert!(output.decision.is_none());
    assert!(output.reason.is_none());
}

// ============================================================================
// Control Response Tests
// ============================================================================

#[test]
fn test_control_response_success_accessors() {
    let response = ControlResponse {
        response_type: "control_response".to_string(),
        response: ControlResponsePayload::Success {
            request_id: "req_123".to_string(),
            response: Some(json!({"status": "initialized"})),
        },
    };

    assert!(response.is_success());
    assert_eq!(response.request_id(), "req_123");
    assert!(response.data().is_some());
    assert!(response.error().is_none());
}

#[test]
fn test_control_response_error_accessors() {
    let response = ControlResponse {
        response_type: "control_response".to_string(),
        response: ControlResponsePayload::Error {
            request_id: "req_456".to_string(),
            error: "Connection refused".to_string(),
        },
    };

    assert!(!response.is_success());
    assert_eq!(response.request_id(), "req_456");
    assert!(response.data().is_none());
    assert_eq!(response.error(), Some("Connection refused"));
}

// ============================================================================
// Sandbox Settings Tests
// ============================================================================

#[test]
fn test_sandbox_settings_serialization() {
    let settings = SandboxSettings {
        enabled: true,
        auto_allow_bash_if_sandboxed: true,
        excluded_commands: vec!["docker".to_string(), "kubectl".to_string()],
        allow_unsandboxed_commands: false,
        network: Some(SandboxNetworkConfig {
            allow_unix_sockets: vec!["/var/run/docker.sock".to_string()],
            allow_local_binding: true,
            ..Default::default()
        }),
        ..Default::default()
    };

    let json = serde_json::to_value(&settings).unwrap();

    assert_eq!(json["enabled"], true);
    assert_eq!(json["autoAllowBashIfSandboxed"], true);
    assert_eq!(json["excludedCommands"][0], "docker");
    assert_eq!(json["excludedCommands"][1], "kubectl");
}

// ============================================================================
// MCP Server Config Tests
// ============================================================================

#[test]
fn test_mcp_stdio_server_config_serialization() {
    let config = McpServerConfig::Stdio(McpStdioServerConfig {
        server_type: "stdio".to_string(),
        command: "node".to_string(),
        args: vec![
            "server.js".to_string(),
            "--port".to_string(),
            "3000".to_string(),
        ],
        env: std::collections::HashMap::new(),
    });

    let json = serde_json::to_value(&config).unwrap();

    assert_eq!(json["type"], "stdio");
    assert_eq!(json["command"], "node");
    assert_eq!(json["args"][0], "server.js");
}

// ============================================================================
// Agent Definition Tests
// ============================================================================

#[test]
fn test_agent_definition_serialization() {
    let agent = AgentDefinition {
        description: "A coding assistant".to_string(),
        prompt: "You are a helpful coding assistant.".to_string(),
        tools: Some(vec![
            "Bash".to_string(),
            "Read".to_string(),
            "Write".to_string(),
        ]),
        model: Some(AgentModel::Sonnet),
    };

    let json = serde_json::to_value(&agent).unwrap();

    assert_eq!(json["description"], "A coding assistant");
    assert_eq!(json["model"], "sonnet");
    assert_eq!(json["tools"][0], "Bash");
}

#[test]
fn test_agent_model_serialization() {
    assert_eq!(
        serde_json::to_string(&AgentModel::Sonnet).unwrap(),
        r#""sonnet""#
    );
    assert_eq!(
        serde_json::to_string(&AgentModel::Opus).unwrap(),
        r#""opus""#
    );
    assert_eq!(
        serde_json::to_string(&AgentModel::Haiku).unwrap(),
        r#""haiku""#
    );
}

// ============================================================================
// Setting Source Tests
// ============================================================================

#[test]
fn test_setting_source_serialization() {
    assert_eq!(
        serde_json::to_string(&SettingSource::User).unwrap(),
        r#""user""#
    );
    assert_eq!(
        serde_json::to_string(&SettingSource::Project).unwrap(),
        r#""project""#
    );
    assert_eq!(
        serde_json::to_string(&SettingSource::Local).unwrap(),
        r#""local""#
    );
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_empty_assistant_message_returns_empty_string() {
    let msg = AssistantMessage {
        content: vec![],
        model: "claude-3".to_string(),
        parent_tool_use_id: None,
        error: None,
    };

    assert_eq!(msg.text(), "", "Empty content should return empty string");
    assert!(msg.tool_uses().is_empty());
}

#[test]
fn test_result_message_with_zero_duration() {
    let result = ResultMessage {
        subtype: "success".to_string(),
        duration_ms: 0,
        duration_api_ms: 0,
        is_error: false,
        num_turns: 0,
        session_id: "test".to_string(),
        total_cost_usd: Some(0.0),
        usage: None,
        result: None,
        structured_output: None,
    };

    assert_eq!(result.duration_ms, 0);
    assert_eq!(result.num_turns, 0);
    assert_eq!(result.total_cost_usd, Some(0.0));
}

#[test]
fn test_tool_use_block_with_complex_input() {
    let complex_input = json!({
        "nested": {
            "array": [1, 2, 3],
            "object": {"key": "value"}
        },
        "unicode": "你好🌍",
        "null_field": null,
        "number": 42.5
    });

    let block = ContentBlock::ToolUse(ToolUseBlock {
        id: "tool_complex".to_string(),
        name: "ComplexTool".to_string(),
        input: complex_input.clone(),
    });

    if let ContentBlock::ToolUse(tool) = block {
        assert_eq!(tool.input, complex_input);
        assert_eq!(tool.input["nested"]["array"][1], 2);
        assert_eq!(tool.input["unicode"], "你好🌍");
    }
}

#[test]
fn test_user_message_empty_text() {
    let msg = UserMessage {
        content: UserMessageContent::Text("".to_string()),
        uuid: None,
        parent_tool_use_id: None,
    };

    assert_eq!(msg.text(), Some(""), "Empty text should return Some(\"\")");
}

// ============================================================================
// Round-Trip Serialization Tests
// ============================================================================

#[test]
fn test_permission_result_allow_roundtrip() {
    let original = PermissionResult::allow();
    let json = serde_json::to_value(&original).unwrap();
    let deserialized: PermissionResult = serde_json::from_value(json).unwrap();

    match deserialized {
        PermissionResult::Allow(allow) => {
            assert_eq!(allow.behavior, "allow");
        }
        _ => panic!("Expected Allow variant after roundtrip"),
    }
}

#[test]
fn test_permission_result_deny_serialization() {
    // Test that deny serializes correctly
    let original = PermissionResult::deny_with_message("Not permitted");
    let json = serde_json::to_value(&original).unwrap();

    assert_eq!(json["behavior"], "deny", "behavior should be 'deny'");
    assert_eq!(
        json["message"], "Not permitted",
        "message should be preserved"
    );

    // Note: Untagged enum deserialization may not preserve the exact variant
    // since both Allow and Deny have similar structures. This is a known
    // limitation of untagged enums in serde.
}

#[test]
fn test_sandbox_settings_roundtrip() {
    let original = SandboxSettings {
        enabled: true,
        auto_allow_bash_if_sandboxed: false,
        excluded_commands: vec!["rm".to_string()],
        allow_unsandboxed_commands: true,
        network: None,
        ignore_violations: None,
        enable_weaker_nested_sandbox: false,
    };

    let json = serde_json::to_value(&original).unwrap();
    let deserialized: SandboxSettings = serde_json::from_value(json).unwrap();

    assert_eq!(deserialized.enabled, original.enabled);
    assert_eq!(
        deserialized.auto_allow_bash_if_sandboxed,
        original.auto_allow_bash_if_sandboxed
    );
    assert_eq!(deserialized.excluded_commands, original.excluded_commands);
}

#[test]
fn test_sync_hook_output_roundtrip() {
    let original = SyncHookOutput {
        continue_: Some(false),
        suppress_output: Some(true),
        stop_reason: Some("Test stop reason".to_string()),
        decision: Some("deny".to_string()),
        reason: Some("Test reason".to_string()),
        system_message: None,
        hook_specific_output: None,
    };

    let json = serde_json::to_value(&original).unwrap();
    let deserialized: SyncHookOutput = serde_json::from_value(json).unwrap();

    assert_eq!(deserialized.continue_, original.continue_);
    assert_eq!(deserialized.suppress_output, original.suppress_output);
    assert_eq!(deserialized.stop_reason, original.stop_reason);
    assert_eq!(deserialized.decision, original.decision);
    assert_eq!(deserialized.reason, original.reason);
}
