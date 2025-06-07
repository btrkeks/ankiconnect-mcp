pub mod stdio;

use async_trait::async_trait;
use anyhow::Result;
use crate::protocol::JsonRpcMessage;

#[async_trait]
pub trait Transport {
    async fn read_message(&mut self) -> Result<JsonRpcMessage>;
    async fn write_message(&mut self, message: JsonRpcMessage) -> Result<()>;
}