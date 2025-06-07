use async_trait::async_trait;
use anyhow::{anyhow, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin, Stdout};
use crate::protocol::JsonRpcMessage;
use crate::transport::Transport;

pub struct StdioTransport {
    reader: BufReader<Stdin>,
    writer: Stdout,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(tokio::io::stdin()),
            writer: tokio::io::stdout(),
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn read_message(&mut self) -> Result<JsonRpcMessage> {
        let mut line = String::new();
        match self.reader.read_line(&mut line).await {
            Ok(0) => Err(anyhow!("EOF reached")),
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    return Err(anyhow!("Empty line received"));
                }
                
                tracing::debug!("Received: {}", line);
                
                let message: JsonRpcMessage = serde_json::from_str(line)
                    .map_err(|e| anyhow!("Failed to parse JSON-RPC message: {}", e))?;
                
                Ok(message)
            }
            Err(e) => Err(anyhow!("Failed to read from stdin: {}", e)),
        }
    }

    async fn write_message(&mut self, message: JsonRpcMessage) -> Result<()> {
        let json = serde_json::to_string(&message)
            .map_err(|e| anyhow!("Failed to serialize JSON-RPC message: {}", e))?;
        
        tracing::debug!("Sending: {}", json);
        
        self.writer.write_all(json.as_bytes()).await
            .map_err(|e| anyhow!("Failed to write to stdout: {}", e))?;
        
        self.writer.write_all(b"\n").await
            .map_err(|e| anyhow!("Failed to write newline to stdout: {}", e))?;
        
        self.writer.flush().await
            .map_err(|e| anyhow!("Failed to flush stdout: {}", e))?;
        
        Ok(())
    }
}