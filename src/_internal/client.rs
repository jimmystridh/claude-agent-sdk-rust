//! Internal client for processing queries.
//!
//! This module provides the core query processing logic used by both
//! the one-shot `query()` function and the streaming `ClaudeClient`.

use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tracing::{debug, info};

use super::query::Query;
use super::transport::{SubprocessTransport, Transport};
use crate::errors::{ClaudeSDKError, Result};
use crate::types::*;

/// A stream that keeps the InternalClient alive while consuming messages.
///
/// This wrapper is used for one-shot queries to ensure the client (and its
/// Query/reader task) stays alive until the stream is fully consumed or dropped.
pub struct ClientStream {
    #[allow(dead_code)]
    client: InternalClient,
    receiver: tokio_stream::wrappers::ReceiverStream<Result<Message>>,
}

impl ClientStream {
    fn new(client: InternalClient, rx: mpsc::Receiver<Result<Message>>) -> Self {
        Self {
            client,
            receiver: tokio_stream::wrappers::ReceiverStream::new(rx),
        }
    }
}

impl Stream for ClientStream {
    type Item = Result<Message>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.receiver).poll_next(cx)
    }
}

/// Internal client for processing Claude queries.
///
/// This is the core implementation that handles communication with the CLI.
/// It's used internally by both the one-shot `query()` function and the
/// streaming `ClaudeClient`.
pub struct InternalClient {
    /// The query handler.
    query: Option<Query>,
    /// Message receiver from the query handler.
    message_rx: Option<mpsc::Receiver<Result<Message>>>,
    /// Options used for this client.
    options: ClaudeAgentOptions,
    /// Whether the client is connected.
    connected: bool,
}

impl InternalClient {
    /// Create a new internal client.
    pub fn new(options: ClaudeAgentOptions) -> Self {
        Self {
            query: None,
            message_rx: None,
            options,
            connected: false,
        }
    }

    /// Validate options before connecting.
    fn validate_options(&self) -> Result<()> {
        // Check for mutually exclusive options
        if self.options.can_use_tool.is_some() && self.options.permission_prompt_tool_name.is_some()
        {
            return Err(ClaudeSDKError::configuration(
                "Cannot specify both 'can_use_tool' and 'permission_prompt_tool_name'",
            ));
        }

        Ok(())
    }

    /// Convert agent definitions to serializable format for the initialize request.
    fn build_agents_dict(options: &ClaudeAgentOptions) -> Option<std::collections::HashMap<String, serde_json::Value>> {
        options.agents.as_ref().map(|agents| {
            agents
                .iter()
                .map(|(name, def)| {
                    let value = serde_json::to_value(def).unwrap_or(serde_json::Value::Null);
                    (name.clone(), value)
                })
                .collect()
        })
    }

    /// Connect to the CLI in streaming mode.
    pub async fn connect(&mut self) -> Result<()> {
        if self.connected {
            return Ok(());
        }

        self.validate_options()?;

        let agents_dict = Self::build_agents_dict(&self.options);

        let mut transport = SubprocessTransport::new(&self.options)?;
        transport.connect().await?;

        // Create query handler with agents
        let (query, message_rx) = Query::new(transport, &self.options, agents_dict);
        self.message_rx = Some(message_rx);
        self.query = Some(query);

        // Start the query handler
        if let Some(ref mut q) = self.query {
            q.start().await?;

            // Initialize the streaming session
            let response = q.initialize().await?;
            debug!("CLI initialized: {:?}", response);
        }

        self.connected = true;
        info!("Connected to Claude CLI");
        Ok(())
    }

    /// Process a one-shot query.
    ///
    /// Always uses streaming mode. Returns a stream of messages from the CLI.
    pub async fn process_query(
        options: ClaudeAgentOptions,
        prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Message>> + Send>>> {
        // Validate options
        if options.can_use_tool.is_some() && options.permission_prompt_tool_name.is_some() {
            return Err(ClaudeSDKError::configuration(
                "Cannot specify both 'can_use_tool' and 'permission_prompt_tool_name'",
            ));
        }

        let has_hooks_or_callbacks =
            options.can_use_tool.is_some() || options.hooks.is_some();

        let mut client = InternalClient::new(options);
        client.connect().await?;
        client.send_message(prompt).await?;

        if has_hooks_or_callbacks {
            // For queries with hooks/callbacks, stdin must stay open for
            // bidirectional control protocol. The reader task will close
            // stdin when it sees the Result message.
            client.set_close_stdin_on_result(true);
        } else {
            // For simple queries, close stdin immediately so the CLI
            // knows no more messages are coming and will exit.
            client.end_input().await?;
        }

        let rx = client
            .take_message_rx()
            .ok_or_else(|| ClaudeSDKError::internal("Message receiver not available"))?;

        Ok(Box::pin(ClientStream::new(client, rx)))
    }

    /// Send a message to the CLI.
    pub async fn send_message(&mut self, message: &str) -> Result<()> {
        let query = self
            .query
            .as_ref()
            .ok_or_else(|| ClaudeSDKError::cli_connection("Client not connected"))?;

        query.send_message(message).await
    }

    /// Enable closing stdin when a Result message is received.
    fn set_close_stdin_on_result(&self, value: bool) {
        if let Some(ref q) = self.query {
            q.set_close_stdin_on_result(value);
        }
    }

    /// Close stdin to signal no more input will be sent.
    pub async fn end_input(&self) -> Result<()> {
        let query = self
            .query
            .as_ref()
            .ok_or_else(|| ClaudeSDKError::cli_connection("Client not connected"))?;

        query.end_input().await
    }

    /// Get the message receiver.
    pub fn take_message_rx(&mut self) -> Option<mpsc::Receiver<Result<Message>>> {
        self.message_rx.take()
    }

    /// Interrupt the current operation.
    pub async fn interrupt(&self) -> Result<()> {
        let query = self
            .query
            .as_ref()
            .ok_or_else(|| ClaudeSDKError::cli_connection("Client not connected"))?;

        query.interrupt().await
    }

    /// Set the permission mode.
    pub async fn set_permission_mode(&self, mode: PermissionMode) -> Result<()> {
        let query = self
            .query
            .as_ref()
            .ok_or_else(|| ClaudeSDKError::cli_connection("Client not connected"))?;

        query.set_permission_mode(mode).await
    }

    /// Set the model.
    pub async fn set_model(&self, model: impl Into<String>) -> Result<()> {
        let query = self
            .query
            .as_ref()
            .ok_or_else(|| ClaudeSDKError::cli_connection("Client not connected"))?;

        query.set_model(model).await
    }

    /// Rewind files to a specific user message.
    pub async fn rewind_files(&self, user_message_id: impl Into<String>) -> Result<()> {
        let query = self
            .query
            .as_ref()
            .ok_or_else(|| ClaudeSDKError::cli_connection("Client not connected"))?;

        query.rewind_files(user_message_id).await
    }

    /// Get server initialization info.
    ///
    /// Returns the initialization response from the CLI, which includes
    /// available commands, output styles, and server capabilities.
    pub async fn get_server_info(&self) -> Option<serde_json::Value> {
        let query = self.query.as_ref()?;
        query.get_server_info().await
    }

    /// Get current MCP server connection status.
    pub async fn get_mcp_status(&self) -> Result<serde_json::Value> {
        let query = self
            .query
            .as_ref()
            .ok_or_else(|| ClaudeSDKError::cli_connection("Client not connected"))?;

        query.get_mcp_status().await
    }

    /// Disconnect from the CLI.
    pub async fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        if let Some(ref mut query) = self.query {
            query.stop().await?;
        }

        self.query = None;
        self.message_rx = None;
        self.connected = false;

        info!("Disconnected from Claude CLI");
        Ok(())
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

impl Drop for InternalClient {
    fn drop(&mut self) {
        // Note: We can't do async cleanup in Drop, but the Query's Drop
        // will abort the reader task, and the transport will kill the process
    }
}

/// Check CLI version and warn if outdated.
pub async fn check_cli_version(cli_path: Option<&std::path::Path>) -> Result<String> {
    use std::process::Stdio;
    use tokio::process::Command;

    let path = cli_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("claude"));

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        Command::new(&path)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output(),
    )
    .await
    .map_err(|_| ClaudeSDKError::timeout(2000))?
    .map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ClaudeSDKError::cli_not_found(format!("CLI not found at {}", path.display()))
        } else {
            ClaudeSDKError::cli_connection_with_source("Failed to run CLI version check", e)
        }
    })?;

    let version_str = String::from_utf8_lossy(&output.stdout);
    let version = version_str
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().last())
        .unwrap_or("unknown")
        .to_string();

    // Check against minimum version
    if let (Ok(found), Ok(required)) = (
        semver::Version::parse(&version),
        semver::Version::parse(crate::MIN_CLI_VERSION),
    ) {
        if found < required {
            tracing::warn!(
                "CLI version {} is below minimum required version {}",
                version,
                crate::MIN_CLI_VERSION
            );
        }
    }

    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_client_new() {
        let options = ClaudeAgentOptions::new();
        let client = InternalClient::new(options);
        assert!(!client.is_connected());
    }

    #[test]
    fn test_validate_options_conflict() {
        use std::sync::Arc;

        let mut options = ClaudeAgentOptions::new();
        options.can_use_tool = Some(Arc::new(|_, _, _| {
            Box::pin(async { PermissionResult::allow() })
        }));
        options.permission_prompt_tool_name = Some("test".to_string());

        let client = InternalClient::new(options);
        assert!(client.validate_options().is_err());
    }
}
