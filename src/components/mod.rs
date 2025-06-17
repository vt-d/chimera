use crate::state::State;
use std::collections::HashMap;
use std::sync::Arc;
use twilight_model::application::interaction::{
    Interaction, message_component::MessageComponentInteractionData,
};

pub mod buttons;

pub type ComponentHandlerFn =
    fn(
        Arc<State>,
        Interaction,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>>;

pub struct ComponentRegistry {
    handlers: HashMap<String, ComponentHandlerFn>,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, custom_id: &str, handler: ComponentHandlerFn) {
        self.handlers.insert(custom_id.to_string(), handler);
    }

    pub async fn handle(
        &self,
        state: Arc<State>,
        interaction: Interaction,
        data: MessageComponentInteractionData,
    ) -> anyhow::Result<()> {
        if let Some(handler) = self.handlers.get(&data.custom_id) {
            (handler)(state, interaction).await
        } else {
            tracing::warn!("No component handler for custom_id: {}", data.custom_id);
            Ok(())
        }
    }
}

pub fn build_registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    crate::components::buttons::register_buttons(&mut reg);
    reg
}
