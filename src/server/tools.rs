use async_trait::async_trait;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use crate::protocol::*;
use crate::server::capabilities::Tool;

pub struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn definition(&self, name: String) -> crate::protocol::Tool {
        crate::protocol::Tool {
            name,
            description: Some("Echoes back the provided text".to_string()),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The text to echo back"
                    }
                },
                "required": ["text"]
            })),
        }
    }

    async fn call(&self, arguments: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let text = arguments.get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing or invalid 'text' parameter"))?;

        Ok(CallToolResult {
            content: vec![ToolResultContent::Text {
                text: format!("Echo: {}", text),
            }],
            is_error: Some(false),
        })
    }
}

pub struct AddTool;

#[async_trait]
impl Tool for AddTool {
    fn definition(&self, name: String) -> crate::protocol::Tool {
        crate::protocol::Tool {
            name,
            description: Some("Adds two numbers together".to_string()),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "a": {
                        "type": "number",
                        "description": "The first number"
                    },
                    "b": {
                        "type": "number",
                        "description": "The second number"
                    }
                },
                "required": ["a", "b"]
            })),
        }
    }

    async fn call(&self, arguments: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let a = arguments.get("a")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Missing or invalid 'a' parameter"))?;

        let b = arguments.get("b")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Missing or invalid 'b' parameter"))?;

        let result = a + b;

        Ok(CallToolResult {
            content: vec![ToolResultContent::Text {
                text: format!("{} + {} = {}", a, b, result),
            }],
            is_error: Some(false),
        })
    }
}

pub struct CurrentTimeTool;

#[async_trait]
impl Tool for CurrentTimeTool {
    fn definition(&self, name: String) -> crate::protocol::Tool {
        crate::protocol::Tool {
            name,
            description: Some("Returns the current timestamp".to_string()),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            })),
        }
    }

    async fn call(&self, _arguments: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(CallToolResult {
            content: vec![ToolResultContent::Text {
                text: format!("Current timestamp: {}", now),
            }],
            is_error: Some(false),
        })
    }
}