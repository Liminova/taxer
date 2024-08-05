use super::text_reaction::TextReactionBuilder;
use crate::{AppError, Context};

use poise::serenity_prelude::{ButtonStyle, Message};

/// Thể hiện sự quan tâm
#[poise::command(prefix_command, slash_command)]
pub async fn qt(
    ctx: Context<'_>,
    #[description = "Message ID of the message to reference"] ref_msg_id: Option<String>,
) -> Result<(), AppError> {
    let text_reaction = TextReactionBuilder::default()
        .set_label("<@USER> đã thể hiện sự quan tâm.")
        .set_also_label("<@USER> cũng thể hiện sự quan tâm.")
        .set_button_label("Thể hiện sự quan tâm của bạn")
        .set_custom_id("qt-btn")
        .set_button_style(ButtonStyle::Success)
        .set_ref_msg_id(ref_msg_id)
        .build();
    text_reaction.slash_command(&ctx).await
}

#[poise::command(context_menu_command = "Quan tâm")]
pub async fn qt_cm(ctx: Context<'_>, ref_msg: Message) -> Result<(), AppError> {
    let text_reaction = TextReactionBuilder::default()
        .set_label("<@USER> đã thể hiện sự quan tâm.")
        .set_also_label("<@USER> cũng thể hiện sự quan tâm.")
        .set_custom_id("qt-btn")
        .set_button_label("Thể hiện sự quan tâm của bạn")
        .set_button_style(ButtonStyle::Success)
        .set_ref_msg_id(Some(ref_msg.id.to_string()))
        .build();
    text_reaction.context_menu_command(&ctx, &ref_msg).await
}
