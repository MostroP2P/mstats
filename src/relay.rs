use nostr_sdk::prelude::*;
use std::time::Duration;

use crate::config::Config;
use crate::models::NostrEvent;

const RELAY_TIMEOUT: Duration = Duration::from_secs(30);

/// Relay client wrapper for fetching Nostr events.
///
/// v1 connects to a single relay with a timeout-based model.
pub struct RelayClient {
    config: Config,
    client: Client,
}

impl RelayClient {
    pub fn new(config: Config) -> Self {
        let client = Client::default();
        RelayClient { config, client }
    }

    /// Ensure we're connected to the relay.
    async fn ensure_connected(&self) -> Result<(), String> {
        let relay_url =
            Url::parse(&self.config.relay_url).map_err(|e| format!("Invalid relay URL: {}", e))?;

        self.client
            .add_relay(relay_url.clone())
            .await
            .map_err(|e| format!("Failed to add relay: {}", e))?;

        self.client.connect().await;

        self.client.wait_for_connection(RELAY_TIMEOUT).await;

        Ok(())
    }

    async fn disconnect(&self) {
        self.client.disconnect().await;
    }

    /// Fetch kind 8383 events from the relay.
    pub async fn fetch_kind_8383_events(&self) -> Result<Vec<NostrEvent>, String> {
        self.ensure_connected().await?;

        let filter = Filter::new().kind(Kind::Custom(8383));
        let events = self
            .client
            .fetch_events(filter, RELAY_TIMEOUT)
            .await
            .map_err(|e| format!("Failed to fetch kind 8383 events: {}", e))?;

        self.disconnect().await;
        Ok(events.iter().map(nostr_event_to_model).collect())
    }

    /// Fetch kind 38383 events filtered by a set of `d` tag values (batched query).
    pub async fn fetch_kind_38383_events(
        &self,
        d_tag_values: &[String],
    ) -> Result<Vec<NostrEvent>, String> {
        if d_tag_values.is_empty() {
            return Ok(vec![]);
        }

        self.ensure_connected().await?;

        // Batch filter: single query with multiple d-tag values
        let filter = Filter::new()
            .kind(Kind::Custom(38383))
            .identifiers(d_tag_values);

        let events = self
            .client
            .fetch_events(filter, RELAY_TIMEOUT)
            .await
            .map_err(|e| format!("Failed to fetch kind 38383 events: {}", e))?;

        self.disconnect().await;
        Ok(events.iter().map(nostr_event_to_model).collect())
    }
}

fn nostr_event_to_model(ev: &nostr_sdk::Event) -> NostrEvent {
    NostrEvent {
        id: ev.id.to_hex(),
        kind: ev.kind.as_u16(),
        pubkey: ev.pubkey.to_hex(),
        created_at: ev.created_at.as_secs(),
        tags: ev.tags.iter().map(|t| t.clone().to_vec()).collect(),
        content: ev.content.clone(),
    }
}
