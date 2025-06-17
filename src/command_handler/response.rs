use chrono::Utc;
use twilight_model::{
    channel::message::{Component, Embed},
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
    util::Timestamp,
};

#[derive(Default, Clone)]
pub struct CommandResponse {
    pub embeds: Vec<Embed>,
    pub content: String,
    pub components: Vec<Component>,
}

impl From<CommandResponse> for InteractionResponse {
    fn from(val: CommandResponse) -> Self {
        InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: if val.content.is_empty() {
                    None
                } else {
                    Some(val.content)
                },
                embeds: if val.embeds.is_empty() {
                    None
                } else {
                    Some(val.embeds)
                },
                components: if val.components.is_empty() {
                    None
                } else {
                    Some(val.components)
                },
                ..Default::default()
            }),
        }
    }
}

#[derive(Default)]
pub struct CommandResponseBuilder {
    embeds: Vec<Embed>,
    content: String,
    components: Vec<Component>,
}

impl CommandResponseBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn embed(mut self, embed: Embed) -> Self {
        self.embeds.push(embed);
        self
    }

    pub fn content<S: Into<String>>(mut self, content: S) -> Self {
        self.content = content.into();
        self
    }

    pub fn component(mut self, component: Component) -> Self {
        self.components.push(component);
        self
    }

    pub fn build(self) -> CommandResponse {
        CommandResponse {
            embeds: self.embeds,
            content: self.content,
            components: self.components,
        }
    }
}

pub fn create_error_response(error: &anyhow::Error) -> CommandResponse {
    let now_utc = Utc::now();
    let timestamp_str = now_utc.to_rfc3339();
    let timestamp = match Timestamp::parse(&timestamp_str) {
        Ok(ts) => Some(ts),
        Err(e) => {
            tracing::warn!(error = ?e, parse_error = ?e, timestamp_str = %timestamp_str, "Failed to parse current timestamp for error embed");
            None
        }
    };

    let embed = Embed {
        title: Some("Command Error".to_string()),
        description: Some(format!(
            "I ran into a problem trying to do that:\n```\n{}```",
            error
        )),
        color: Some(0xdd7878),
        timestamp,
        kind: "rich".to_string(),
        author: None,
        fields: Vec::new(),
        footer: None,
        image: None,
        provider: None,
        thumbnail: None,
        url: None,
        video: None,
    };

    CommandResponse {
        embeds: vec![embed],
        content: String::new(),
        components: Vec::new(),
    }
}
