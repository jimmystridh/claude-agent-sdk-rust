//! Tests for MCP (Model Context Protocol) tool support.
//!
//! These tests verify the MCP types, builders, and tool execution.

#![cfg(feature = "mcp")]

use claude_agents_sdk::mcp::{
    create_sdk_mcp_server, McpSdkServerConfig, SdkMcpTool, ToolContent, ToolInputSchema, ToolResult,
};
use serde_json::json;

// ============================================================================
// ToolContent Tests
// ============================================================================

#[test]
fn test_tool_content_text_creation() {
    let content = ToolContent::text("Hello, World!");
    match content {
        ToolContent::Text { text } => {
            assert_eq!(text, "Hello, World!");
        }
        _ => panic!("Expected text content"),
    }
}

#[test]
fn test_tool_content_text_empty() {
    let content = ToolContent::text("");
    match content {
        ToolContent::Text { text } => {
            assert_eq!(text, "");
        }
        _ => panic!("Expected text content"),
    }
}

#[test]
fn test_tool_content_image_creation() {
    let content = ToolContent::image("base64data==", "image/png");
    match content {
        ToolContent::Image { data, mime_type } => {
            assert_eq!(data, "base64data==");
            assert_eq!(mime_type, "image/png");
        }
        _ => panic!("Expected image content"),
    }
}

#[test]
fn test_tool_content_serialization() {
    let text = ToolContent::text("Test");
    let serialized = serde_json::to_value(&text).unwrap();
    assert_eq!(serialized["type"], "text");
    assert_eq!(serialized["text"], "Test");

    let image = ToolContent::image("data", "image/jpeg");
    let serialized = serde_json::to_value(&image).unwrap();
    assert_eq!(serialized["type"], "image");
    assert_eq!(serialized["data"], "data");
    assert_eq!(serialized["mimeType"], "image/jpeg");
}

#[test]
fn test_tool_content_deserialization() {
    let json = json!({
        "type": "text",
        "text": "Deserialized"
    });
    let content: ToolContent = serde_json::from_value(json).unwrap();
    match content {
        ToolContent::Text { text } => assert_eq!(text, "Deserialized"),
        _ => panic!("Expected text content"),
    }
}

// ============================================================================
// ToolResult Tests
// ============================================================================

#[test]
fn test_tool_result_text() {
    let result = ToolResult::text("Success!");
    assert_eq!(result.content.len(), 1);
    assert!(result.is_error.is_none());
}

#[test]
fn test_tool_result_error() {
    let result = ToolResult::error("Something went wrong");
    assert_eq!(result.is_error, Some(true));
    assert_eq!(result.content.len(), 1);

    if let ToolContent::Text { text } = &result.content[0] {
        assert_eq!(text, "Something went wrong");
    } else {
        panic!("Expected text content in error result");
    }
}

#[test]
fn test_tool_result_with_multiple_content() {
    let content = vec![
        ToolContent::text("Line 1"),
        ToolContent::text("Line 2"),
        ToolContent::text("Line 3"),
    ];
    let result = ToolResult::with_content(content);
    assert_eq!(result.content.len(), 3);
    assert!(result.is_error.is_none());
}

#[test]
fn test_tool_result_serialization() {
    let result = ToolResult::text("OK");
    let serialized = serde_json::to_value(&result).unwrap();

    assert!(serialized["content"].is_array());
    assert_eq!(serialized["content"][0]["type"], "text");
    assert_eq!(serialized["content"][0]["text"], "OK");
    // is_error should be skipped when None
    assert!(serialized.get("is_error").is_none());
}

#[test]
fn test_tool_result_error_serialization() {
    let result = ToolResult::error("Failed");
    let serialized = serde_json::to_value(&result).unwrap();

    assert_eq!(serialized["is_error"], true);
}

// ============================================================================
// ToolInputSchema Tests
// ============================================================================

#[test]
fn test_input_schema_object() {
    let schema = ToolInputSchema::object();
    assert_eq!(schema.schema_type, "object");
    assert!(schema.properties.is_empty());
    assert!(schema.required.is_empty());
}

#[test]
fn test_input_schema_string_property() {
    let schema = ToolInputSchema::object().string_property("name", "The user's name");

    assert!(schema.properties.contains_key("name"));
    let prop = &schema.properties["name"];
    assert_eq!(prop["type"], "string");
    assert_eq!(prop["description"], "The user's name");
}

#[test]
fn test_input_schema_number_property() {
    let schema = ToolInputSchema::object().number_property("age", "The user's age");

    assert!(schema.properties.contains_key("age"));
    let prop = &schema.properties["age"];
    assert_eq!(prop["type"], "number");
    assert_eq!(prop["description"], "The user's age");
}

#[test]
fn test_input_schema_boolean_property() {
    let schema = ToolInputSchema::object().boolean_property("active", "Whether the user is active");

    assert!(schema.properties.contains_key("active"));
    let prop = &schema.properties["active"];
    assert_eq!(prop["type"], "boolean");
    assert_eq!(prop["description"], "Whether the user is active");
}

#[test]
fn test_input_schema_required_property() {
    let schema = ToolInputSchema::object()
        .string_property("name", "Name")
        .required_property("name");

    assert!(schema.required.contains(&"name".to_string()));
}

#[test]
fn test_input_schema_builder_chaining() {
    let schema = ToolInputSchema::object()
        .string_property("first_name", "First name")
        .string_property("last_name", "Last name")
        .number_property("age", "Age")
        .boolean_property("active", "Is active")
        .required_property("first_name")
        .required_property("last_name");

    assert_eq!(schema.properties.len(), 4);
    assert_eq!(schema.required.len(), 2);
}

#[test]
fn test_input_schema_serialization() {
    let schema = ToolInputSchema::object()
        .string_property("query", "Search query")
        .required_property("query");

    let serialized = serde_json::to_value(&schema).unwrap();

    assert_eq!(serialized["type"], "object");
    assert!(serialized["properties"]["query"]["type"].is_string());
    assert!(serialized["required"]
        .as_array()
        .unwrap()
        .contains(&json!("query")));
}

// ============================================================================
// SdkMcpTool Tests
// ============================================================================

#[test]
fn test_sdk_mcp_tool_creation() {
    let tool = SdkMcpTool::new(
        "greet",
        "Greet a user",
        ToolInputSchema::object()
            .string_property("name", "Name to greet")
            .required_property("name"),
        |_input| async { ToolResult::text("Hello!") },
    );

    assert_eq!(tool.name, "greet");
    assert_eq!(tool.description, "Greet a user");
    assert!(tool.input_schema.properties.contains_key("name"));
}

#[test]
fn test_sdk_mcp_tool_debug() {
    let tool = SdkMcpTool::new("test", "Test tool", ToolInputSchema::object(), |_| async {
        ToolResult::text("ok")
    });

    let debug_str = format!("{:?}", tool);
    assert!(debug_str.contains("test"));
    assert!(debug_str.contains("Test tool"));
}

#[tokio::test]
async fn test_sdk_mcp_tool_handler_execution() {
    let tool = SdkMcpTool::new(
        "add",
        "Add numbers",
        ToolInputSchema::object()
            .number_property("a", "First")
            .number_property("b", "Second"),
        |input| async move {
            let a = input.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b = input.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
            ToolResult::text(format!("{}", a + b))
        },
    );

    let input = json!({"a": 5.0, "b": 3.0});
    let result = (tool.handler)(input).await;

    assert_eq!(result.content.len(), 1);
    if let ToolContent::Text { text } = &result.content[0] {
        assert_eq!(text, "8");
    } else {
        panic!("Expected text content");
    }
}

#[tokio::test]
async fn test_sdk_mcp_tool_error_handling() {
    let tool = SdkMcpTool::new(
        "divide",
        "Divide numbers",
        ToolInputSchema::object()
            .number_property("a", "Dividend")
            .number_property("b", "Divisor"),
        |input| async move {
            let a = input.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b = input.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
            if b == 0.0 {
                return ToolResult::error("Division by zero");
            }
            ToolResult::text(format!("{}", a / b))
        },
    );

    // Test division by zero
    let input = json!({"a": 10.0, "b": 0.0});
    let result = (tool.handler)(input).await;
    assert_eq!(result.is_error, Some(true));

    // Test normal division
    let input = json!({"a": 10.0, "b": 2.0});
    let result = (tool.handler)(input).await;
    assert!(result.is_error.is_none());
}

// ============================================================================
// McpSdkServerConfig Tests
// ============================================================================

#[test]
fn test_mcp_sdk_server_config_serialization() {
    let config = McpSdkServerConfig {
        server_type: "sdk".to_string(),
        name: "test-server".to_string(),
        version: "1.0.0".to_string(),
    };

    let serialized = serde_json::to_value(&config).unwrap();
    assert_eq!(serialized["type"], "sdk");
    assert_eq!(serialized["name"], "test-server");
    assert_eq!(serialized["version"], "1.0.0");
}

#[test]
fn test_mcp_sdk_server_config_deserialization() {
    let json = json!({
        "type": "sdk",
        "name": "my-server",
        "version": "2.0.0"
    });

    let config: McpSdkServerConfig = serde_json::from_value(json).unwrap();
    assert_eq!(config.server_type, "sdk");
    assert_eq!(config.name, "my-server");
    assert_eq!(config.version, "2.0.0");
}

// ============================================================================
// create_sdk_mcp_server Tests
// ============================================================================

#[test]
fn test_create_sdk_mcp_server_empty() {
    let (config, tools) = create_sdk_mcp_server("empty-server", "1.0.0", vec![]);

    assert_eq!(config.server_type, "sdk");
    assert_eq!(config.name, "empty-server");
    assert_eq!(config.version, "1.0.0");
    assert!(tools.is_empty());
}

#[test]
fn test_create_sdk_mcp_server_with_tools() {
    let tool1 = SdkMcpTool::new(
        "tool1",
        "First tool",
        ToolInputSchema::object(),
        |_| async { ToolResult::text("1") },
    );
    let tool2 = SdkMcpTool::new(
        "tool2",
        "Second tool",
        ToolInputSchema::object(),
        |_| async { ToolResult::text("2") },
    );

    let (config, tools) = create_sdk_mcp_server("multi-tool", "2.0.0", vec![tool1, tool2]);

    assert_eq!(config.name, "multi-tool");
    assert_eq!(config.version, "2.0.0");
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].name, "tool1");
    assert_eq!(tools[1].name, "tool2");
}

// ============================================================================
// Integration-style Tests (MCP tool workflows)
// ============================================================================

#[tokio::test]
async fn test_calculator_tools_workflow() {
    // Create a set of calculator tools
    let add = SdkMcpTool::new(
        "add",
        "Add two numbers",
        ToolInputSchema::object()
            .number_property("a", "First number")
            .number_property("b", "Second number")
            .required_property("a")
            .required_property("b"),
        |input| async move {
            let a = input.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b = input.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
            ToolResult::text(format!("{}", a + b))
        },
    );

    let multiply = SdkMcpTool::new(
        "multiply",
        "Multiply two numbers",
        ToolInputSchema::object()
            .number_property("a", "First number")
            .number_property("b", "Second number"),
        |input| async move {
            let a = input.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b = input.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
            ToolResult::text(format!("{}", a * b))
        },
    );

    let (config, tools) = create_sdk_mcp_server("calculator", "1.0.0", vec![add, multiply]);

    assert_eq!(config.name, "calculator");
    assert_eq!(tools.len(), 2);

    // Execute add
    let add_result = (tools[0].handler)(json!({"a": 10, "b": 5})).await;
    if let ToolContent::Text { text } = &add_result.content[0] {
        assert_eq!(text, "15");
    }

    // Execute multiply
    let mul_result = (tools[1].handler)(json!({"a": 4, "b": 7})).await;
    if let ToolContent::Text { text } = &mul_result.content[0] {
        assert_eq!(text, "28");
    }
}

#[tokio::test]
async fn test_string_processing_tools() {
    let uppercase = SdkMcpTool::new(
        "uppercase",
        "Convert to uppercase",
        ToolInputSchema::object()
            .string_property("text", "Text to convert")
            .required_property("text"),
        |input| async move {
            let text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
            ToolResult::text(text.to_uppercase())
        },
    );

    let reverse = SdkMcpTool::new(
        "reverse",
        "Reverse a string",
        ToolInputSchema::object()
            .string_property("text", "Text to reverse")
            .required_property("text"),
        |input| async move {
            let text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
            ToolResult::text(text.chars().rev().collect::<String>())
        },
    );

    let (_, tools) = create_sdk_mcp_server("string-utils", "1.0.0", vec![uppercase, reverse]);

    // Test uppercase
    let result = (tools[0].handler)(json!({"text": "hello"})).await;
    if let ToolContent::Text { text } = &result.content[0] {
        assert_eq!(text, "HELLO");
    }

    // Test reverse
    let result = (tools[1].handler)(json!({"text": "hello"})).await;
    if let ToolContent::Text { text } = &result.content[0] {
        assert_eq!(text, "olleh");
    }
}

#[tokio::test]
async fn test_tool_with_missing_input() {
    let tool = SdkMcpTool::new(
        "greet",
        "Greet someone",
        ToolInputSchema::object()
            .string_property("name", "Name")
            .required_property("name"),
        |input| async move {
            let name = input
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("stranger");
            ToolResult::text(format!("Hello, {}!", name))
        },
    );

    // Missing name should use default
    let result = (tool.handler)(json!({})).await;
    if let ToolContent::Text { text } = &result.content[0] {
        assert_eq!(text, "Hello, stranger!");
    }
}

#[tokio::test]
async fn test_tool_with_complex_output() {
    let tool = SdkMcpTool::new(
        "stats",
        "Calculate statistics",
        ToolInputSchema::object(),
        |_input| async move {
            ToolResult::with_content(vec![
                ToolContent::text("Count: 10"),
                ToolContent::text("Sum: 55"),
                ToolContent::text("Average: 5.5"),
            ])
        },
    );

    let result = (tool.handler)(json!({})).await;
    assert_eq!(result.content.len(), 3);
}
