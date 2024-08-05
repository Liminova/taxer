use crate::{AppError, Context};

/// get help.
#[poise::command(prefix_command, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "specific command to show help about"] command: Option<String>,
) -> Result<(), AppError> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "\
Type t!help <command> for extra help on a command.
You can edit your message to the bot and it will edit its response.",
        ephemeral: true,
        ..Default::default()
    };

    poise::builtins::help(ctx, command.as_deref(), config).await?;

    Ok(())
}
