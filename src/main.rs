use anyhow::Result;
use tracing_subscriber;
use ankiconnect_mcp::{McpServer, StdioTransport};
use ankiconnect_mcp::server::anki_tools::ListDecksTool;
use ankiconnect_mcp::server::resources::StaticTextResource;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let mut server = McpServer::new();

    server.add_tool("list_decks".to_string(), Box::new(ListDecksTool));

    server.add_resource(
        "anki://connection-help".to_string(),
        Box::new(StaticTextResource::new(
            "connection-help".to_string(),
            "Help for connecting to AnkiConnect".to_string(),
            r#"AnkiConnect Setup Instructions:

1. Install Anki desktop application
2. Install AnkiConnect plugin:
   - Go to Tools > Add-ons in Anki
   - Click "Get Add-ons..."
   - Enter code: 2055492159
   - Click OK and restart Anki
3. Verify AnkiConnect is running:
   - Visit http://localhost:8765 in your browser
   - You should see "AnkiConnect v.X" displayed
4. Keep Anki open while using this MCP server

Troubleshooting:
- If connection fails, ensure Anki is running
- Check that AnkiConnect plugin is enabled
- Verify no firewall is blocking port 8765
- Try restarting Anki if the plugin was just installed"#.to_string(),
        )),
    );

    server.add_resource(
        "anki://about".to_string(),
        Box::new(StaticTextResource::new(
            "about".to_string(),
            "About the AnkiConnect MCP Server".to_string(),
            r#"AnkiConnect MCP Server

This Model Context Protocol (MCP) server provides integration with Anki flashcard software through the AnkiConnect plugin.

Available Tools:
- list_decks: Retrieves all Anki decks with statistics, hierarchy, and card information

Features:
- Comprehensive deck information including statistics
- Hierarchical deck structure visualization  
- Card counts and availability status
- Error handling with helpful troubleshooting messages
- Real-time connection status reporting

Requirements:
- Anki desktop application
- AnkiConnect plugin (code: 2055492159)
- Anki running with AnkiConnect accessible on localhost:8765

This server implements MCP specification 2025-03-26 and provides a foundation for building AI assistants that can interact with your Anki flashcard collection."#.to_string(),
        )),
    );

    let transport = StdioTransport::new();
    server.run(transport).await?;

    Ok(())
}