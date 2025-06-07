pub mod capabilities;
pub mod tools;
pub mod resources;
pub mod anki_tools;

use anyhow::Result;
use std::collections::HashMap;
use crate::protocol::*;
use crate::transport::Transport;
use crate::server::capabilities::{Tool, Resource};

pub struct McpServer {
    initialized: bool,
    tools: HashMap<String, Box<dyn Tool + Send + Sync>>,
    resources: HashMap<String, Box<dyn Resource + Send + Sync>>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            initialized: false,
            tools: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    pub fn add_tool(&mut self, name: String, tool: Box<dyn Tool + Send + Sync>) {
        self.tools.insert(name, tool);
    }

    pub fn add_resource(&mut self, uri: String, resource: Box<dyn Resource + Send + Sync>) {
        self.resources.insert(uri, resource);
    }

    pub async fn run<T: Transport>(&mut self, mut transport: T) -> Result<()> {
        tracing::info!("Starting MCP server");

        loop {
            match transport.read_message().await {
                Ok(message) => {
                    if let Some(response) = self.handle_message(message).await? {
                        transport.write_message(response).await?;
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading message: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_message(&mut self, message: JsonRpcMessage) -> Result<Option<JsonRpcMessage>> {
        match message.content {
            MessageContent::Request(request) => {
                let response = self.handle_request(request).await;
                Ok(Some(response))
            }
            MessageContent::Notification(notification) => {
                self.handle_notification(notification).await;
                Ok(None)
            }
            MessageContent::Response(_) => {
                tracing::warn!("Received unexpected response message");
                Ok(None)
            }
        }
    }

    async fn handle_request(&mut self, request: Request) -> JsonRpcMessage {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "tools/list" => self.handle_list_tools().await,
            "tools/call" => self.handle_call_tool(request.params).await,
            "resources/list" => self.handle_list_resources().await,
            "resources/read" => self.handle_read_resource(request.params).await,
            _ => Err(McpError::method_not_found()),
        };

        let response = match result {
            Ok(result) => Response {
                id: request.id,
                result: Some(result),
                error: None,
            },
            Err(error) => Response {
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: error.code,
                    message: error.message,
                    data: error.data,
                }),
            },
        };

        JsonRpcMessage {
            jsonrpc: "2.0".to_string(),
            content: MessageContent::Response(response),
        }
    }

    async fn handle_notification(&mut self, notification: Notification) {
        match notification.method.as_str() {
            "initialized" => {
                tracing::info!("Client sent initialized notification");
            }
            _ => {
                tracing::warn!("Unknown notification method: {}", notification.method);
            }
        }
    }

    async fn handle_initialize(&mut self, params: Option<serde_json::Value>) -> Result<serde_json::Value, McpError> {
        let _init_request: InitializeRequest = params
            .ok_or_else(|| McpError::invalid_params())?
            .try_into()
            .map_err(|_| McpError::invalid_params())?;

        self.initialized = true;

        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: ServerCapabilities {
                logging: None,
                prompts: None,
                resources: Some(ResourcesCapability {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
            },
            server_info: ServerInfo {
                name: "ankiconnect-mcp".to_string(),
                version: "0.1.0".to_string(),
            },
        };

        serde_json::to_value(result).map_err(|_| McpError::internal_error())
    }

    async fn handle_list_tools(&self) -> Result<serde_json::Value, McpError> {
        if !self.initialized {
            return Err(McpError::custom(-32002, "Server not initialized".to_string()));
        }

        let tools: Vec<crate::protocol::Tool> = self.tools
            .iter()
            .map(|(name, tool)| tool.definition(name.clone()))
            .collect();

        let result = ListToolsResult { tools };
        serde_json::to_value(result).map_err(|_| McpError::internal_error())
    }

    async fn handle_call_tool(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, McpError> {
        if !self.initialized {
            return Err(McpError::custom(-32002, "Server not initialized".to_string()));
        }

        let call_request: CallToolRequest = params
            .ok_or_else(|| McpError::invalid_params())?
            .try_into()
            .map_err(|_| McpError::invalid_params())?;

        let tool = self.tools.get(&call_request.name)
            .ok_or_else(|| McpError::custom(-32601, format!("Tool '{}' not found", call_request.name)))?;

        let result = tool.call(call_request.arguments.unwrap_or_default()).await
            .map_err(|e| McpError::custom(-32603, format!("Tool execution failed: {}", e)))?;

        serde_json::to_value(result).map_err(|_| McpError::internal_error())
    }

    async fn handle_list_resources(&self) -> Result<serde_json::Value, McpError> {
        if !self.initialized {
            return Err(McpError::custom(-32002, "Server not initialized".to_string()));
        }

        let resources: Vec<crate::protocol::Resource> = self.resources
            .iter()
            .map(|(uri, resource)| resource.definition(uri.clone()))
            .collect();

        let result = ListResourcesResult { resources };
        serde_json::to_value(result).map_err(|_| McpError::internal_error())
    }

    async fn handle_read_resource(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, McpError> {
        if !self.initialized {
            return Err(McpError::custom(-32002, "Server not initialized".to_string()));
        }

        let read_request: ReadResourceRequest = params
            .ok_or_else(|| McpError::invalid_params())?
            .try_into()
            .map_err(|_| McpError::invalid_params())?;

        let resource = self.resources.get(&read_request.uri)
            .ok_or_else(|| McpError::custom(-32601, format!("Resource '{}' not found", read_request.uri)))?;

        let contents = resource.read().await
            .map_err(|e| McpError::custom(-32603, format!("Resource read failed: {}", e)))?;

        let result = ReadResourceResult { contents };
        serde_json::to_value(result).map_err(|_| McpError::internal_error())
    }
}

impl TryFrom<serde_json::Value> for InitializeRequest {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl TryFrom<serde_json::Value> for CallToolRequest {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl TryFrom<serde_json::Value> for ReadResourceRequest {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}