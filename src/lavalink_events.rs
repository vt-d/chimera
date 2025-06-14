use lavalink_rs::{client::LavalinkClient, hook, model::events};

#[hook]
pub async fn ready_event(client: LavalinkClient, session_id: String, event: &events::Ready) {
    client.delete_all_player_contexts().await.unwrap();
    tracing::info!("{:?} -> {:?}", session_id, event);
}
