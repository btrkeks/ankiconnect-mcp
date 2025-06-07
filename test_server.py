#!/usr/bin/env python3
"""
Simple test script to verify the MCP server functionality.
"""
import json
import subprocess
import sys

def test_mcp_server():
    # Start the MCP server
    process = subprocess.Popen(
        ['cargo', 'run'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0
    )
    
    def send_message(message):
        json_msg = json.dumps(message)
        print(f"Sending: {json_msg}")
        process.stdin.write(json_msg + '\n')
        process.stdin.flush()
        
        response = process.stdout.readline()
        if response:
            print(f"Received: {response.strip()}")
            return json.loads(response.strip())
        return None
    
    try:
        # Test 1: Initialize the server
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "roots": {"list_changed": False},
                    "sampling": {}
                },
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        }
        
        response = send_message(init_request)
        assert response is not None, "No response to initialize"
        assert response.get("result"), "Initialize failed"
        print("✓ Initialize test passed")
        
        # Send initialized notification (no response expected)
        initialized_notification = {
            "jsonrpc": "2.0",
            "method": "initialized"
        }
        json_msg = json.dumps(initialized_notification)
        print(f"Sending: {json_msg}")
        process.stdin.write(json_msg + '\n')
        process.stdin.flush()
        print("✓ Initialized notification sent")
        
        # Test 2: List tools
        list_tools_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }
        
        response = send_message(list_tools_request)
        assert response is not None, "No response to tools/list"
        assert "result" in response, "tools/list failed"
        tools = response["result"]["tools"]
        assert len(tools) == 1, f"Expected 1 tool, got {len(tools)}"
        print("✓ List tools test passed")
        
        # Test 3: Call list_decks tool
        list_decks_request = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "list_decks",
                "arguments": {}
            }
        }
        
        response = send_message(list_decks_request)
        assert response is not None, "No response to list_decks tool"
        assert "result" in response, "list_decks tool failed"
        content = response["result"]["content"][0]["text"]
        assert "decks" in content or "Error connecting to Anki" in content, f"Unexpected list_decks response: {content}"
        print("✓ List decks tool test passed")
        
        # Test 4: List resources
        list_resources_request = {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "resources/list"
        }
        
        response = send_message(list_resources_request)
        assert response is not None, "No response to resources/list"
        assert "result" in response, "resources/list failed"
        resources = response["result"]["resources"]
        assert len(resources) == 2, f"Expected 2 resources, got {len(resources)}"
        print("✓ List resources test passed")
        
        # Test 5: Read connection help resource
        read_resource_request = {
            "jsonrpc": "2.0",
            "id": 5,
            "method": "resources/read",
            "params": {
                "uri": "anki://connection-help"
            }
        }
        
        response = send_message(read_resource_request)
        assert response is not None, "No response to read resource"
        assert "result" in response, "Read resource failed"
        content = response["result"]["contents"][0]["text"]
        assert "AnkiConnect Setup Instructions" in content, f"Unexpected resource content: {content}"
        print("✓ Read connection help resource test passed")
        
        # Test 6: Read about resource
        read_about_request = {
            "jsonrpc": "2.0",
            "id": 6,
            "method": "resources/read",
            "params": {
                "uri": "anki://about"
            }
        }
        
        response = send_message(read_about_request)
        assert response is not None, "No response to read about resource"
        assert "result" in response, "Read about resource failed"
        content = response["result"]["contents"][0]["text"]
        assert "AnkiConnect MCP Server" in content, f"Unexpected about resource content: {content}"
        print("✓ Read about resource test passed")
        
        print("\n🎉 All tests passed! MCP server is working correctly.")
        
    except Exception as e:
        print(f"Test failed: {e}")
        return False
    finally:
        process.terminate()
        process.wait()
    
    return True

if __name__ == "__main__":
    success = test_mcp_server()
    sys.exit(0 if success else 1)
