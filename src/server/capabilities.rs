use async_trait::async_trait;
use anyhow::Result;
use std::collections::HashMap;
use crate::protocol::*;

#[async_trait]
pub trait Tool {
    fn definition(&self, name: String) -> crate::protocol::Tool;
    async fn call(&self, arguments: HashMap<String, serde_json::Value>) -> Result<CallToolResult>;
}

#[async_trait]
pub trait Resource {
    fn definition(&self, uri: String) -> crate::protocol::Resource;
    async fn read(&self) -> Result<Vec<ResourceContent>>;
}