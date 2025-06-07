use async_trait::async_trait;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use ankiconnect_rs::{AnkiClient, Deck};
use crate::protocol::*;
use crate::server::capabilities::Tool;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeckInfo {
    pub id: String,
    pub name: String,
    pub deck_type: String, // "root" or "subdeck"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_name: Option<String>,
    pub base_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statistics: Option<DeckStatistics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_count: Option<usize>,
    pub cards_available: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeckStatistics {
    pub new_count: u32,
    pub learn_count: u32,
    pub review_count: u32,
    pub total_in_deck: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeckHierarchyNode {
    pub name: String,
    pub id: String,
    pub children: Vec<DeckHierarchyNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ankiconnect_version: Option<String>,
    pub total_decks: usize,
    pub timestamp: u64,
    pub connection_successful: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListDecksResponse {
    pub decks: Vec<DeckInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hierarchy: Option<Vec<DeckHierarchyNode>>,
    pub connection_info: ConnectionInfo,
}

pub struct ListDecksTool;

impl ListDecksTool {
    fn convert_deck(&self, deck: &Deck) -> DeckInfo {
        DeckInfo {
            id: deck.id().0.to_string(),
            name: deck.name().to_string(),
            deck_type: if deck.is_subdeck() { "subdeck" } else { "root" }.to_string(),
            parent_name: deck.parent_name().map(|s| s.to_string()),
            base_name: deck.base_name().to_string(),
            statistics: None, // Will be filled separately
            card_count: None, // Will be filled separately
            cards_available: false,
        }
    }

    fn convert_tree_node(&self, node: &ankiconnect_rs::client::request::DeckTreeNode) -> DeckHierarchyNode {
        DeckHierarchyNode {
            name: node.name.clone(),
            id: node.id.to_string(),
            children: node.children.iter().map(|child| self.convert_tree_node(child)).collect(),
        }
    }

    async fn fetch_deck_data(&self) -> Result<ListDecksResponse> {
        // Create AnkiConnect client
        let client = AnkiClient::new();
        
        // Check connection and version
        let version = client.version().map_err(|e| {
            anyhow!("Failed to connect to AnkiConnect. Please ensure Anki is running and AnkiConnect plugin is installed. Error: {}", e)
        })?;

        tracing::info!("Connected to AnkiConnect version: {}", version);

        // Get all decks
        let decks = client.decks().get_all().map_err(|e| {
            anyhow!("Failed to retrieve decks from Anki: {}", e)
        })?;

        let mut deck_infos = Vec::new();

        // Process each deck
        for deck in &decks {
            let mut deck_info = self.convert_deck(deck);

            // Try to get statistics for this deck
            match client.decks().get_stat(deck.name()) {
                Ok(stats) => {
                    deck_info.statistics = Some(DeckStatistics {
                        new_count: stats.new_count,
                        learn_count: stats.learn_count,
                        review_count: stats.review_count,
                        total_in_deck: stats.total_in_deck,
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to get statistics for deck '{}': {}", deck.name(), e);
                }
            }

            // Try to get card count for this deck
            match client.decks().get_cards_in_deck(deck.name()) {
                Ok(cards) => {
                    deck_info.card_count = Some(cards.len());
                    deck_info.cards_available = true;
                }
                Err(e) => {
                    tracing::warn!("Failed to get cards for deck '{}': {}", deck.name(), e);
                    deck_info.cards_available = false;
                }
            }

            deck_infos.push(deck_info);
        }

        // Try to get deck hierarchy
        let hierarchy = match client.decks().get_tree() {
            Ok(tree) => Some(tree.iter().map(|node| self.convert_tree_node(node)).collect()),
            Err(e) => {
                tracing::warn!("Failed to get deck hierarchy: {}", e);
                None
            }
        };

        let connection_info = ConnectionInfo {
            ankiconnect_version: Some(version.to_string()),
            total_decks: decks.len(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            connection_successful: true,
        };

        Ok(ListDecksResponse {
            decks: deck_infos,
            hierarchy,
            connection_info,
        })
    }
}

#[async_trait]
impl Tool for ListDecksTool {
    fn definition(&self, name: String) -> crate::protocol::Tool {
        crate::protocol::Tool {
            name,
            description: Some("Lists all Anki decks with statistics, hierarchy, and card information".to_string()),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            })),
        }
    }

    async fn call(&self, _arguments: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        match self.fetch_deck_data().await {
            Ok(response) => {
                let json_response = serde_json::to_string_pretty(&response)
                    .map_err(|e| anyhow!("Failed to serialize response: {}", e))?;

                Ok(CallToolResult {
                    content: vec![ToolResultContent::Text {
                        text: json_response,
                    }],
                    is_error: Some(false),
                })
            }
            Err(e) => {
                let error_message = format!(
                    "Error connecting to Anki: {}\n\nTroubleshooting:\n1. Ensure Anki is running\n2. Install AnkiConnect plugin (code: 2055492159)\n3. Verify AnkiConnect is accessible on localhost:8765\n4. Restart Anki if the plugin was just installed",
                    e
                );

                Ok(CallToolResult {
                    content: vec![ToolResultContent::Text {
                        text: error_message,
                    }],
                    is_error: Some(true),
                })
            }
        }
    }
}