pub mod lyrics;
pub mod pause;
pub mod skip;

pub fn register_buttons(reg: &mut crate::components::ComponentRegistry) {
    reg.register("skip", |state, interaction| {
        Box::pin(crate::components::buttons::skip::skip_button_handler(
            state,
            interaction,
        ))
    });
    reg.register("pause", |state, interaction| {
        Box::pin(crate::components::buttons::pause::pause_button_handler(
            state,
            interaction,
        ))
    });
    reg.register("lyrics", |state, interaction| {
        Box::pin(crate::components::buttons::lyrics::lyrics_button_handler(
            state,
            interaction,
        ))
    });
}
