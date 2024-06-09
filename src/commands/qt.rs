use poise::{
    serenity_prelude::{ButtonStyle, CreateMessage, Message},
    CreateReply,
};

use crate::{Context, Error};

const QUANTAM_BTN_ID: &str = "qt-btn";
const QUANTAM_BTN_LABEL: &str = "Thể hiện sự quan tâm của bạn";
const QUANTAM_TEXT: &str = "<@USER> đã thể hiện sự quan tâm.";
const QUANTAM_ALSO_TEXT: &str = "<@USER> cũng thể hiện sự quan tâm.";

/// Thể hiện sự quan tâm
#[poise::command(prefix_command, slash_command)]
pub async fn qt(
    ctx: Context<'_>,
    #[description = "Message ID of the message to reference"] ref_msg_id: Option<String>,
) -> Result<(), Error> {
    let also_btn = super::also_btn::AlsoButton::new(
        QUANTAM_BTN_ID,
        QUANTAM_BTN_LABEL,
        ButtonStyle::Success,
        QUANTAM_ALSO_TEXT,
    );

    // get the ref msg given the msg id from the given arg
    let ref_msg_arg = match ref_msg_id.as_ref().and_then(|s| s.parse::<u64>().ok()) {
        Some(ref_msg) => match ctx
            .channel_id()
            .message(&ctx.serenity_context(), ref_msg)
            .await
        {
            Ok(ref_msg) => Some(ref_msg),
            Err(_) => None,
        },
        _ => None,
    };

    // to reference the replied msg, not the command msg itself
    if let Context::Prefix(pctx) = ctx {
        if let Some(ref_msg) = pctx.msg.referenced_message.as_ref() {
            let ref_msg = *ref_msg.clone();
            pctx.msg
                .channel_id
                .send_message(
                    &ctx.serenity_context(),
                    CreateMessage::default()
                        .content(QUANTAM_TEXT.replace("USER", &ctx.author().id.to_string()))
                        .components(also_btn.create(Some(ref_msg.id)))
                        .reference_message(&ref_msg),
                )
                .await?;
            return also_btn.handler(ctx).await;
        }
    }

    match ref_msg_arg {
        // if ref-msg-id provided, reply it
        Some(ref_msg) => {
            ctx.send(CreateReply::default().content("cứ từ từ").ephemeral(true))
                .await?;

            ctx.channel_id()
                .send_message(
                    &ctx.serenity_context(),
                    CreateMessage::default()
                        .content(QUANTAM_ALSO_TEXT.replace("USER", &ctx.author().id.to_string()))
                        .components(also_btn.create(Some(ref_msg.id)))
                        .reference_message(&ref_msg),
                )
                .await?;
        }
        // else reply the user's command
        None => {
            ctx.send(
                CreateReply::default()
                    .content(QUANTAM_TEXT.replace("USER", &ctx.author().id.to_string()))
                    .components(also_btn.create(None)),
            )
            .await?;
        }
    };

    also_btn.handler(ctx).await
}

#[poise::command(context_menu_command = "Quan tâm")]
pub async fn qt_cm(ctx: Context<'_>, ref_msg: Message) -> Result<(), Error> {
    let also_btn = super::also_btn::AlsoButton::new(
        QUANTAM_BTN_ID,
        QUANTAM_BTN_LABEL,
        ButtonStyle::Success,
        QUANTAM_ALSO_TEXT,
    );

    ctx.send(CreateReply::default().content("cứ từ từ").ephemeral(true))
        .await?;

    ctx.channel_id()
        .send_message(
            &ctx.serenity_context(),
            CreateMessage::default()
                .content(QUANTAM_TEXT.replace("USER", &ctx.author().id.to_string()))
                .components(also_btn.create(Some(ref_msg.id)))
                .reference_message(&ref_msg),
        )
        .await?;

    also_btn.handler(ctx).await
}
