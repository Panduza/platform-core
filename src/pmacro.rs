#[macro_export]
macro_rules! spawn_loop {
    ($task_name:literal, $device:ident, $body:expr) => {
        $device
            .spawn($task_name, async move {
                loop {
                    $body
                }
            })
            .await
    };
}

#[macro_export]
macro_rules! on_command {
    ($attribute:ident, $body:expr) => {
        $attribute.wait_commands_then($body).await?
    };
}

#[macro_export]
macro_rules! spawn_on_command {
    ($task_name:literal, $device:ident, $attribute:ident, $callback:expr) => {
        $device
            .spawn($task_name, async move {
                loop {
                    $attribute.wait_commands_then($callback).await?
                }
            })
            .await
    };
}
