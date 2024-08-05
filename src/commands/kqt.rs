use super::text_reaction::TextReactionBuilder;
use crate::{AppError, Context};

use poise::serenity_prelude::{ButtonStyle, Message};

/// Thể hiện sự không quan tâm
#[poise::command(prefix_command, slash_command)]
pub async fn kqt(
    ctx: Context<'_>,
    #[description = "Message ID of the message to reference"] ref_msg_id: Option<String>,
) -> Result<(), AppError> {
    let text_reaction = TextReactionBuilder::default()
        .set_label("<@USER> đã đéo hỏi.")
        .set_also_label("<@USER> cũng đã đéo hỏi.")
        .set_button_label("Yêu cầu tìm ra người đã hỏi")
        .set_custom_id("kqt-btn")
        .set_button_style(ButtonStyle::Danger)
        .set_ref_msg_id(ref_msg_id)
        .build();

    text_reaction.slash_command(&ctx).await
}

#[poise::command(context_menu_command = "Không quan tâm")]
pub async fn kqt_cm(ctx: Context<'_>, ref_msg: Message) -> Result<(), AppError> {
    let text_reaction = TextReactionBuilder::default()
        .set_label("<@USER> đã đéo hỏi.")
        .set_also_label("<@USER> cũng đã đéo hỏi.")
        .set_custom_id("kqt-btn")
        .set_button_label("Yêu cầu tìm ra người đã hỏi")
        .set_button_style(ButtonStyle::Danger)
        .set_ref_msg_id(Some(ref_msg.id.to_string()))
        .build();

    text_reaction.context_menu_command(&ctx, &ref_msg).await
}
