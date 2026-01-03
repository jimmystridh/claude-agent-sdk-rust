//! Tests for Claude SDK client options and configuration.
//!
//! This file consolidates all ClaudeAgentOptions builder tests
//! and message type discrimination tests.

use claude_agents_sdk::{
    AgentDefinition, AssistantMessage, ClaudeAgentOptions, ContentBlock, McpServerConfig,
    McpServersConfig, McpStdioServerConfig, Message, PermissionMode, ResultMessage,
    SandboxNetworkConfig, SandboxSettings, SettingSource, SystemPromptConfig, SystemPromptPreset,
    TextBlock, ToolsConfig, ToolsPreset,
};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// ClaudeAgentOptions Default State Tests
// ============================================================================

#[test]
fn test_new_options_has_all_fields_unset() {
    let options = ClaudeAgentOptions::new();

    // Core fields
    assert!(options.model.is_none(), "model should be None by default");
    assert!(
        options.system_prompt.is_none(),
        "system_prompt should be None by default"
    );
    assert!(
        options.permission_mode.is_none(),
        "permission_mode should be None by default"
    );
    assert!(
        options.max_turns.is_none(),
        "max_turns should be None by default"
    );
    assert!(
        options.max_budget_usd.is_none(),
        "max_budget_usd should be None by default"
    );

    // Tool configuration
    assert!(options.tools.is_none(), "tools should be None by default");
    assert!(
        options.allowed_tools.is_empty(),
        "allowed_tools should be empty by default"
    );
    assert!(
        options.disallowed_tools.is_empty(),
        "disallowed_tools should be empty by default"
    );

    // Session management
    assert!(
        !options.continue_conversation,
        "continue_conversation should be false by default"
    );
    assert!(options.resume.is_none(), "resume should be None by default");
    assert!(
        !options.fork_session,
        "fork_session should be false by default"
    );

    // Callbacks
    assert!(
        options.can_use_tool.is_none(),
        "can_use_tool should be None by default"
    );
    assert!(options.hooks.is_none(), "hooks should be None by default");

    // Streaming
    assert!(
        !options.include_partial_messages,
        "include_partial_messages should be false by default"
    );
}

// ============================================================================
// Builder Chain Tests
// ============================================================================

#[test]
fn test_builder_chain_sets_all_fields_correctly() {
    let options = ClaudeAgentOptions::new()
        .with_model("claude-sonnet-4-5")
        .with_max_turns(10)
        .with_permission_mode(PermissionMode::AcceptEdits)
        .with_system_prompt("Be helpful and concise")
        .with_cwd("/test/path")
        .with_allowed_tools(vec!["Read".to_string(), "Write".to_string()])
        .with_timeout_secs(120)
        .with_partial_messages();

    assert_eq!(
        options.model,
        Some("claude-sonnet-4-5".to_string()),
        "model should match set value"
    );
    assert_eq!(
        options.max_turns,
        Some(10),
        "max_turns should match set value"
    );
    assert_eq!(
        options.permission_mode,
        Some(PermissionMode::AcceptEdits),
        "permission_mode should match set value"
    );
    assert_eq!(
        options.cwd,
        Some(PathBuf::from("/test/path")),
        "cwd should match set value"
    );
    assert_eq!(
        options.allowed_tools,
        vec!["Read", "Write"],
        "allowed_tools should match set value"
    );
    assert_eq!(
        options.timeout_secs,
        Some(120),
        "timeout_secs should match set value"
    );
    assert!(
        options.include_partial_messages,
        "include_partial_messages should be true"
    );

    match &options.system_prompt {
        Some(SystemPromptConfig::Text(text)) => {
            assert_eq!(
                text, "Be helpful and concise",
                "system_prompt text should match"
            )
        }
        _ => panic!(
            "Expected text system prompt, got {:?}",
            options.system_prompt
        ),
    }
}

#[test]
fn test_builder_methods_are_idempotent() {
    let options1 = ClaudeAgentOptions::new()
        .with_model("claude-3")
        .with_max_turns(5);

    let options2 = ClaudeAgentOptions::new()
        .with_model("claude-3")
        .with_max_turns(5);

    assert_eq!(
        options1.model, options2.model,
        "Same builder calls should produce same model"
    );
    assert_eq!(
        options1.max_turns, options2.max_turns,
        "Same builder calls should produce same max_turns"
    );
}

// ============================================================================
// System Prompt Configuration Tests
// ============================================================================

#[test]
fn test_system_prompt_text_configuration() {
    let options = ClaudeAgentOptions::new().with_system_prompt("Custom instructions");

    match options.system_prompt {
        Some(SystemPromptConfig::Text(text)) => {
            assert_eq!(text, "Custom instructions");
        }
        other => panic!("Expected SystemPromptConfig::Text, got {:?}", other),
    }
}

#[test]
fn test_system_prompt_preset_without_append() {
    let mut options = ClaudeAgentOptions::new();
    options.system_prompt = Some(SystemPromptConfig::Preset(SystemPromptPreset {
        preset_type: "preset".to_string(),
        preset: "claude_code".to_string(),
        append: None,
    }));

    match options.system_prompt {
        Some(SystemPromptConfig::Preset(preset)) => {
            assert_eq!(preset.preset, "claude_code");
            assert!(preset.append.is_none(), "append should be None");
        }
        other => panic!("Expected SystemPromptConfig::Preset, got {:?}", other),
    }
}

#[test]
fn test_system_prompt_preset_with_append() {
    let mut options = ClaudeAgentOptions::new();
    options.system_prompt = Some(SystemPromptConfig::Preset(SystemPromptPreset {
        preset_type: "preset".to_string(),
        preset: "claude_code".to_string(),
        append: Some("Be concise.".to_string()),
    }));

    match options.system_prompt {
        Some(SystemPromptConfig::Preset(preset)) => {
            assert_eq!(preset.preset, "claude_code");
            assert_eq!(preset.append, Some("Be concise.".to_string()));
        }
        other => panic!("Expected SystemPromptConfig::Preset, got {:?}", other),
    }
}

// ============================================================================
// Tools Configuration Tests
// ============================================================================

#[test]
fn test_tools_list_configuration() {
    let mut options = ClaudeAgentOptions::new();
    options.tools = Some(ToolsConfig::List(vec![
        "Read".to_string(),
        "Write".to_string(),
        "Bash".to_string(),
    ]));

    match &options.tools {
        Some(ToolsConfig::List(tools)) => {
            assert_eq!(tools.len(), 3);
            assert!(tools.contains(&"Read".to_string()));
            assert!(tools.contains(&"Write".to_string()));
            assert!(tools.contains(&"Bash".to_string()));
        }
        other => panic!("Expected ToolsConfig::List, got {:?}", other),
    }
}

#[test]
fn test_tools_empty_list_is_valid() {
    let mut options = ClaudeAgentOptions::new();
    options.tools = Some(ToolsConfig::List(vec![]));

    match &options.tools {
        Some(ToolsConfig::List(tools)) => {
            assert!(tools.is_empty(), "Empty tools list should be allowed");
        }
        other => panic!("Expected ToolsConfig::List, got {:?}", other),
    }
}

#[test]
fn test_tools_preset_configuration() {
    let mut options = ClaudeAgentOptions::new();
    options.tools = Some(ToolsConfig::Preset(ToolsPreset {
        preset_type: "preset".to_string(),
        preset: "claude_code".to_string(),
    }));

    match &options.tools {
        Some(ToolsConfig::Preset(preset)) => {
            assert_eq!(preset.preset, "claude_code");
        }
        other => panic!("Expected ToolsConfig::Preset, got {:?}", other),
    }
}

#[test]
fn test_allowed_and_disallowed_tools_both_set() {
    let mut options = ClaudeAgentOptions::new();
    options.allowed_tools = vec!["Read".to_string(), "Write".to_string()];
    options.disallowed_tools = vec!["Bash".to_string()];

    assert_eq!(options.allowed_tools, vec!["Read", "Write"]);
    assert_eq!(options.disallowed_tools, vec!["Bash"]);
}

// ============================================================================
// Session Management Tests
// ============================================================================

#[test]
fn test_session_continuation_configuration() {
    let mut options = ClaudeAgentOptions::new();
    options.continue_conversation = true;
    options.resume = Some("session-abc123".to_string());

    assert!(options.continue_conversation);
    assert_eq!(options.resume, Some("session-abc123".to_string()));
}

#[test]
fn test_fork_session_configuration() {
    let mut options = ClaudeAgentOptions::new();
    options.resume = Some("session-xyz789".to_string());
    options.fork_session = true;

    assert!(options.fork_session);
    assert_eq!(options.resume, Some("session-xyz789".to_string()));
}

// ============================================================================
// Model Configuration Tests
// ============================================================================

#[test]
fn test_model_configuration() {
    let options = ClaudeAgentOptions::new().with_model("claude-opus-4-5");
    assert_eq!(options.model, Some("claude-opus-4-5".to_string()));
}

#[test]
fn test_fallback_model_configuration() {
    let mut options = ClaudeAgentOptions::new();
    options.model = Some("opus".to_string());
    options.fallback_model = Some("sonnet".to_string());

    assert_eq!(options.model, Some("opus".to_string()));
    assert_eq!(options.fallback_model, Some("sonnet".to_string()));
}

#[test]
fn test_max_thinking_tokens_configuration() {
    let mut options = ClaudeAgentOptions::new();
    options.max_thinking_tokens = Some(5000);

    assert_eq!(options.max_thinking_tokens, Some(5000));
}

// ============================================================================
// Directory and Path Configuration Tests
// ============================================================================

#[test]
fn test_add_dirs_configuration() {
    let mut options = ClaudeAgentOptions::new();
    options.add_dirs = vec![
        PathBuf::from("/path/to/dir1"),
        PathBuf::from("/path/to/dir2"),
    ];

    assert_eq!(options.add_dirs.len(), 2);
    assert!(options.add_dirs.contains(&PathBuf::from("/path/to/dir1")));
    assert!(options.add_dirs.contains(&PathBuf::from("/path/to/dir2")));
}

#[test]
fn test_cwd_configuration() {
    let options = ClaudeAgentOptions::new().with_cwd("/custom/working/dir");
    assert_eq!(options.cwd, Some(PathBuf::from("/custom/working/dir")));
}

#[test]
fn test_cli_path_configuration() {
    let mut options = ClaudeAgentOptions::new();
    options.cli_path = Some(PathBuf::from("/usr/local/bin/claude"));

    assert_eq!(
        options.cli_path,
        Some(PathBuf::from("/usr/local/bin/claude"))
    );
}

// ============================================================================
// Environment and Extra Args Configuration Tests
// ============================================================================

#[test]
fn test_env_vars_configuration() {
    let mut options = ClaudeAgentOptions::new();
    let mut env = HashMap::new();
    env.insert("MY_VAR".to_string(), "my_value".to_string());
    env.insert("ANOTHER_VAR".to_string(), "another_value".to_string());
    options.env = env;

    assert_eq!(options.env.len(), 2);
    assert_eq!(options.env.get("MY_VAR"), Some(&"my_value".to_string()));
    assert_eq!(
        options.env.get("ANOTHER_VAR"),
        Some(&"another_value".to_string())
    );
}

#[test]
fn test_extra_args_configuration() {
    let mut options = ClaudeAgentOptions::new();
    let mut extra_args = HashMap::new();
    extra_args.insert("new-flag".to_string(), Some("value".to_string()));
    extra_args.insert("boolean-flag".to_string(), None);
    options.extra_args = extra_args;

    assert_eq!(options.extra_args.len(), 2);
    assert_eq!(
        options.extra_args.get("new-flag"),
        Some(&Some("value".to_string()))
    );
    assert_eq!(options.extra_args.get("boolean-flag"), Some(&None));
}

// ============================================================================
// Settings Configuration Tests
// ============================================================================

#[test]
fn test_settings_string_configuration() {
    let mut options = ClaudeAgentOptions::new();
    options.settings = Some(r#"{"permissions": {"allow": ["Bash(ls:*)"]}}"#.to_string());

    let settings = options.settings.as_ref().expect("settings should be set");
    assert!(
        settings.contains("permissions"),
        "settings should contain 'permissions'"
    );
}

#[test]
fn test_setting_sources_configuration() {
    let mut options = ClaudeAgentOptions::new();
    options.setting_sources = Some(vec![SettingSource::User, SettingSource::Project]);

    let sources = options
        .setting_sources
        .as_ref()
        .expect("sources should be set");
    assert_eq!(sources.len(), 2);
    assert!(sources.contains(&SettingSource::User));
    assert!(sources.contains(&SettingSource::Project));
}

// ============================================================================
// MCP Server Configuration Tests
// ============================================================================

#[test]
fn test_mcp_servers_map_configuration() {
    let mut mcp_servers = HashMap::new();
    mcp_servers.insert(
        "test-server".to_string(),
        McpServerConfig::Stdio(McpStdioServerConfig {
            server_type: "stdio".to_string(),
            command: "/path/to/server".to_string(),
            args: vec!["--option".to_string(), "value".to_string()],
            env: HashMap::new(),
        }),
    );

    let mut options = ClaudeAgentOptions::new();
    options.mcp_servers = McpServersConfig::Map(mcp_servers);

    match &options.mcp_servers {
        McpServersConfig::Map(servers) => {
            assert!(servers.contains_key("test-server"));
        }
        other => panic!("Expected McpServersConfig::Map, got {:?}", other),
    }
}

#[test]
fn test_mcp_servers_path_configuration() {
    let mut options = ClaudeAgentOptions::new();
    options.mcp_servers = McpServersConfig::Path(PathBuf::from("/path/to/mcp-config.json"));

    match &options.mcp_servers {
        McpServersConfig::Path(path) => {
            assert_eq!(path, &PathBuf::from("/path/to/mcp-config.json"));
        }
        other => panic!("Expected McpServersConfig::Path, got {:?}", other),
    }
}

// ============================================================================
// Sandbox Configuration Tests
// ============================================================================

#[test]
fn test_sandbox_configuration() {
    let mut options = ClaudeAgentOptions::new();
    options.sandbox = Some(SandboxSettings {
        enabled: true,
        auto_allow_bash_if_sandboxed: true,
        excluded_commands: vec!["docker".to_string()],
        allow_unsandboxed_commands: true,
        network: Some(SandboxNetworkConfig {
            allow_unix_sockets: vec!["/var/run/docker.sock".to_string()],
            allow_all_unix_sockets: false,
            allow_local_binding: true,
            http_proxy_port: None,
            socks_proxy_port: None,
        }),
        ignore_violations: None,
        enable_weaker_nested_sandbox: false,
    });

    let sandbox = options.sandbox.as_ref().expect("sandbox should be set");
    assert!(sandbox.enabled);
    assert!(sandbox.auto_allow_bash_if_sandboxed);
    assert!(sandbox.network.is_some());
}

// ============================================================================
// Agent Configuration Tests
// ============================================================================

#[test]
fn test_agents_configuration() {
    let mut agents = HashMap::new();
    agents.insert(
        "test-agent".to_string(),
        AgentDefinition {
            description: "A test agent".to_string(),
            prompt: "You are a test agent".to_string(),
            tools: Some(vec!["Read".to_string()]),
            model: None,
        },
    );

    let mut options = ClaudeAgentOptions::new();
    options.agents = Some(agents);

    let agents = options.agents.as_ref().expect("agents should be set");
    assert!(agents.contains_key("test-agent"));
    let agent = agents.get("test-agent").unwrap();
    assert_eq!(agent.description, "A test agent");
}

// ============================================================================
// Miscellaneous Configuration Tests
// ============================================================================

#[test]
fn test_partial_messages_configuration() {
    let options = ClaudeAgentOptions::new().with_partial_messages();
    assert!(options.include_partial_messages);
}

#[test]
fn test_user_configuration() {
    let mut options = ClaudeAgentOptions::new();
    options.user = Some("claude-user".to_string());

    assert_eq!(options.user, Some("claude-user".to_string()));
}

#[test]
fn test_max_buffer_size_configuration() {
    let mut options = ClaudeAgentOptions::new();
    options.max_buffer_size = Some(1024 * 1024); // 1MB

    assert_eq!(options.max_buffer_size, Some(1024 * 1024));
}

// ============================================================================
// Message Type Discrimination Tests
// ============================================================================

#[test]
fn test_message_is_assistant_returns_true_for_assistant_message() {
    let msg = Message::Assistant(AssistantMessage {
        content: vec![ContentBlock::Text(TextBlock {
            text: "Hello".to_string(),
        })],
        model: "claude-3".to_string(),
        parent_tool_use_id: None,
        error: None,
    });

    assert!(
        msg.is_assistant(),
        "is_assistant() should return true for Assistant message"
    );
    assert!(
        !msg.is_result(),
        "is_result() should return false for Assistant message"
    );
}

#[test]
fn test_message_is_result_returns_true_for_result_message() {
    let msg = Message::Result(ResultMessage {
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

    assert!(
        msg.is_result(),
        "is_result() should return true for Result message"
    );
    assert!(
        !msg.is_assistant(),
        "is_assistant() should return false for Result message"
    );
}

#[test]
fn test_message_as_assistant_returns_some_for_assistant_message() {
    let msg = Message::Assistant(AssistantMessage {
        content: vec![ContentBlock::Text(TextBlock {
            text: "Hello".to_string(),
        })],
        model: "claude-3".to_string(),
        parent_tool_use_id: None,
        error: None,
    });

    let asst = msg.as_assistant();
    assert!(
        asst.is_some(),
        "as_assistant() should return Some for Assistant message"
    );
    assert_eq!(
        asst.unwrap().text(),
        "Hello",
        "as_assistant() should return the correct message"
    );
}

#[test]
fn test_message_as_assistant_returns_none_for_non_assistant_message() {
    let msg = Message::Result(ResultMessage {
        subtype: "success".to_string(),
        duration_ms: 100,
        duration_api_ms: 80,
        is_error: false,
        num_turns: 1,
        session_id: "test".to_string(),
        total_cost_usd: Some(0.001),
        usage: None,
        result: None,
        structured_output: None,
    });

    assert!(
        msg.as_assistant().is_none(),
        "as_assistant() should return None for Result message"
    );
}

#[test]
fn test_message_as_result_returns_some_for_result_message() {
    let msg = Message::Result(ResultMessage {
        subtype: "success".to_string(),
        duration_ms: 100,
        duration_api_ms: 80,
        is_error: false,
        num_turns: 1,
        session_id: "test".to_string(),
        total_cost_usd: Some(0.001),
        usage: None,
        result: None,
        structured_output: None,
    });

    let result = msg.as_result();
    assert!(
        result.is_some(),
        "as_result() should return Some for Result message"
    );
    assert_eq!(
        result.unwrap().total_cost_usd,
        Some(0.001),
        "as_result() should return the correct message"
    );
}

#[test]
fn test_message_as_result_returns_none_for_non_result_message() {
    let msg = Message::Assistant(AssistantMessage {
        content: vec![ContentBlock::Text(TextBlock {
            text: "Hello".to_string(),
        })],
        model: "claude-3".to_string(),
        parent_tool_use_id: None,
        error: None,
    });

    assert!(
        msg.as_result().is_none(),
        "as_result() should return None for Assistant message"
    );
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_empty_model_string_is_preserved() {
    let options = ClaudeAgentOptions::new().with_model("");
    assert_eq!(
        options.model,
        Some("".to_string()),
        "Empty model string should be preserved"
    );
}

#[test]
fn test_zero_max_turns_is_valid() {
    let mut options = ClaudeAgentOptions::new();
    options.max_turns = Some(0);
    assert_eq!(
        options.max_turns,
        Some(0),
        "Zero max_turns should be allowed"
    );
}

#[test]
fn test_zero_timeout_disables_timeout() {
    let options = ClaudeAgentOptions::new().with_timeout_secs(0);
    assert_eq!(
        options.timeout_secs,
        Some(0),
        "Zero timeout should be allowed (disables timeout)"
    );
}

#[test]
fn test_large_max_turns_value() {
    let mut options = ClaudeAgentOptions::new();
    options.max_turns = Some(u32::MAX);
    assert_eq!(
        options.max_turns,
        Some(u32::MAX),
        "Large max_turns should be allowed"
    );
}

#[test]
fn test_negative_budget_is_representable() {
    // Note: Negative budgets probably shouldn't be allowed semantically,
    // but we test that the type can represent them
    let mut options = ClaudeAgentOptions::new();
    options.max_budget_usd = Some(-1.0);
    assert!(
        options.max_budget_usd.unwrap() < 0.0,
        "Negative budget value is representable"
    );
}

#[test]
fn test_whitespace_only_system_prompt() {
    let options = ClaudeAgentOptions::new().with_system_prompt("   ");
    match options.system_prompt {
        Some(SystemPromptConfig::Text(text)) => {
            assert_eq!(text, "   ", "Whitespace-only prompt should be preserved");
        }
        other => panic!("Expected text system prompt, got {:?}", other),
    }
}

#[test]
fn test_unicode_in_system_prompt() {
    let options = ClaudeAgentOptions::new().with_system_prompt("你好世界 🌍 مرحبا");
    match options.system_prompt {
        Some(SystemPromptConfig::Text(text)) => {
            assert_eq!(text, "你好世界 🌍 مرحبا", "Unicode should be preserved");
        }
        other => panic!("Expected text system prompt, got {:?}", other),
    }
}

#[test]
fn test_very_long_allowed_tools_list() {
    let tools: Vec<String> = (0..1000).map(|i| format!("Tool{}", i)).collect();
    let options = ClaudeAgentOptions::new().with_allowed_tools(tools.clone());
    assert_eq!(
        options.allowed_tools.len(),
        1000,
        "Large tools list should be handled"
    );
}

#[test]
fn test_duplicate_tools_in_allowed_tools() {
    let options = ClaudeAgentOptions::new().with_allowed_tools(vec![
        "Read".to_string(),
        "Read".to_string(),
        "Write".to_string(),
    ]);
    assert_eq!(
        options.allowed_tools.len(),
        3,
        "Duplicate tools should be preserved (not deduplicated)"
    );
}

#[test]
fn test_overlapping_allowed_and_disallowed_tools() {
    let mut options = ClaudeAgentOptions::new();
    options.allowed_tools = vec!["Bash".to_string()];
    options.disallowed_tools = vec!["Bash".to_string()];

    // Both can be set - validation would happen at a higher level
    assert!(options.allowed_tools.contains(&"Bash".to_string()));
    assert!(options.disallowed_tools.contains(&"Bash".to_string()));
}
