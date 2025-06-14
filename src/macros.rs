#[macro_export]
macro_rules! match_command {
    ($command_name:expr, $state:expr, $interaction:expr, $data:expr, {$($cmd:literal => $handler:ty),* $(,)?}) => {
        match $command_name {
            $(|
                $cmd => {
                    <$handler>::execute_slash_command($state, $interaction, $data).await?
                }
            )*
            name => {
                tracing::warn!("Unknown slash command: {}", name);
            }
        }
    };
    ($command_name:expr, $state:expr, $message:expr, $arguments:expr, $prefix_string:expr, {$($cmd:literal => $handler:ty),* $(,)?}) => {
        match $command_name {
            $(|
                $cmd => {
                    run_prefix_command::<$handler>($state, $message, $arguments, $prefix_string).await?
                }
            )*
            name => {
                tracing::debug!(
                    "Unknown prefix command: {} from user: {}",
                    name,
                    $message.author.name
                );
            }
        }
    };
}