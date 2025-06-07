use async_trait::async_trait;
use anyhow::Result;
use crate::protocol::*;
use crate::server::capabilities::Resource;

pub struct StaticTextResource {
    name: String,
    description: String,
    content: String,
}

impl StaticTextResource {
    pub fn new(name: String, description: String, content: String) -> Self {
        Self {
            name,
            description,
            content,
        }
    }
}

#[async_trait]
impl Resource for StaticTextResource {
    fn definition(&self, uri: String) -> crate::protocol::Resource {
        crate::protocol::Resource {
            uri,
            name: self.name.clone(),
            description: Some(self.description.clone()),
            mime_type: Some("text/plain".to_string()),
        }
    }

    async fn read(&self) -> Result<Vec<ResourceContent>> {
        Ok(vec![ResourceContent::Text {
            uri: format!("static://{}", self.name),
            text: self.content.clone(),
            mime_type: Some("text/plain".to_string()),
        }])
    }
}