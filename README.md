# AnkiConnect MCP Server

⚠️ **Work in Progress** ⚠️
This project is in early development. Expect breaking changes, incomplete features, and potential instability.

A Model Context Protocol (MCP) server implementation in Rust that provides seamless integration with Anki flashcard software through the AnkiConnect plugin.
It is a wrapper for the [ankiconnect-rs](https://crates.io/crates/ankiconnect-rs) crate.

## Features

There aren't many tools implemented yet, but more are to come.

### Tools
- **list_decks**: Lists all Anki decks with detailed information including:
  - Deck names, IDs, and hierarchical structure  
  - Statistics (new, learning, review, total card counts)
  - Parent-child deck relationships
  - Card availability status
  - Connection status and AnkiConnect version

### Resources
- **anki://connection-help**: Setup instructions and troubleshooting guide for AnkiConnect
- **anki://about**: Information about the server and its capabilities

## Prerequisites

### System Requirements
- **Rust 1.70+** with Cargo
- **Anki Desktop Application** (version 2.1+)
- **AnkiConnect Plugin** (addon code: 2055492159)

### AnkiConnect Setup
1. Install Anki desktop application
2. Install AnkiConnect plugin:
   - Go to Tools > Add-ons in Anki
   - Click "Get Add-ons..."
   - Enter code: `2055492159`
   - Click OK and restart Anki
3. Verify AnkiConnect is running:
   - Visit `http://localhost:8765` in your browser
   - You should see "AnkiConnect v.X" displayed
4. Keep Anki open while using this MCP server

## Building and Running

### Build
```bash
cargo build
```

### Run
```bash
cargo run
```

The server communicates via stdin/stdout using JSON-RPC 2.0 messages and requires Anki to be running with AnkiConnect enabled.

## Configuration

To use this MCP server with Claude Desktop or other MCP clients, add the following configuration to your MCP settings file:

```json
{
  "mcpServers": {
    "anki": {
      "type": "stdio",
      "command": "cargo",
      "args": [
        "run",
        "--manifest-path",
        "/path/to/your/ankiconnect-mcp/Cargo.toml"
      ],
      "env": {}
    }
  }
}
```

Replace `/path/to/your/ankiconnect-mcp/Cargo.toml` with the actual path to this project's Cargo.toml file on your system.

## Architecture

The server is built with a modular architecture:

- `src/protocol/`: MCP protocol message definitions and error types
- `src/transport/`: Transport layer (stdio implementation)  
- `src/server/`: Core server logic, capabilities, tools, and resources
- `src/server/anki_tools.rs`: AnkiConnect integration and deck management
- `src/main.rs`: Entry point and server setup

## Testing

### Manual Testing
You can test the server manually by sending JSON-RPC messages:

```bash
# Run comprehensive tests (requires Anki running with AnkiConnect)
cargo run < test_input.txt

# Test single tool call
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_decks","arguments":{}}}' | cargo run
```

### Example Usage

1. **Initialize the server:**
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocol_version":"2025-03-26","capabilities":{"roots":{"list_changed":false},"sampling":{}},"client_info":{"name":"test-client","version":"1.0.0"}}}
```

2. **List available tools:**
```json
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
```

3. **List all Anki decks:**
```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_decks","arguments":{}}}
```

4. **List available resources:**
```json
{"jsonrpc":"2.0","id":4,"method":"resources/list"}
```

5. **Get connection help:**
```json
{"jsonrpc":"2.0","id":5,"method":"resources/read","params":{"uri":"anki://connection-help"}}
```

### Example Response

The `list_decks` tool returns comprehensive deck information:

```json
{
  "decks": [
    {
      "id": "1632125614258",
      "name": "Japanese::Grammar",
      "deck_type": "subdeck",
      "parent_name": "Japanese",
      "base_name": "Grammar",
      "statistics": {
        "new_count": 15,
        "learn_count": 3,
        "review_count": 42,
        "total_in_deck": 238
      },
      "card_count": 238,
      "cards_available": true
    }
  ],
  "connection_info": {
    "ankiconnect_version": "6",
    "total_decks": 12,
    "timestamp": 1649323748,
    "connection_successful": true
  }
}
```

## Error Handling

The server provides graceful error handling for common scenarios:

- **Anki Not Running**: Returns helpful error message with setup instructions
- **AnkiConnect Plugin Missing**: Provides installation guidance  
- **Network Issues**: Offers troubleshooting steps
- **Partial Failures**: Continues operation when individual deck operations fail

Example error response when Anki is not running:
```json
{
  "content": [
    {
      "type": "text", 
      "text": "Error connecting to Anki: Failed to connect to AnkiConnect...\n\nTroubleshooting:\n1. Ensure Anki is running\n2. Install AnkiConnect plugin (code: 2055492159)\n3. Verify AnkiConnect is accessible on localhost:8765\n4. Restart Anki if the plugin was just installed"
    }
  ],
  "is_error": true
}
```

## Protocol Compliance

This server implements the Model Context Protocol specification version 2024-11-05 with support for:

- Server initialization and capability negotiation
- Tool listing and execution  
- Resource listing and reading
- JSON-RPC 2.0 message format
- Stdio transport
- Comprehensive error reporting

## Future Enhancement Ideas

- **Additional Tools**: Add tools for creating cards, managing study sessions, importing/exporting
- **Deck Filtering**: Support filtering decks by criteria (name patterns, card counts, etc.)
- **Real-time Updates**: Implement notifications for deck changes
- **Study Analytics**: Provide detailed study statistics and progress tracking
- **Batch Operations**: Support bulk card operations and deck management
