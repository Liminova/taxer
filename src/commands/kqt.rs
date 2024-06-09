use poise::{
    serenity_prelude::{ButtonStyle, CreateMessage, Message},
    CreateReply,
};

use crate::{Context, Error};

const KQT_BTN_ID: &str = "kqt-btn";
const KQT_BTN_LABEL: &str = "Yêu cầu tìm ra người đã hỏi";
const KQT_TEXT: &str = "<@USER> đã đéo hỏi.";
const KQT_ALSO_TEXT: &str = KQT_TEXT;

/// Thể hiện sự không quan tâm
#[poise::command(prefix_command, slash_command)]
pub async fn kqt(
    ctx: Context<'_>,
    #[description = "Message ID of the message to reference"] ref_msg_id: Option<String>,
) -> Result<(), Error> {
    let also_btn = super::also_btn::AlsoButton::new(
        KQT_BTN_ID,
        KQT_BTN_LABEL,
        ButtonStyle::Danger,
        KQT_ALSO_TEXT,
    );

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

    if let Context::Prefix(pctx) = ctx {
        if let Some(ref_msg) = pctx.msg.referenced_message.as_ref() {
            pctx.msg
                .channel_id
                .send_message(
                    &ctx.serenity_context(),
                    CreateMessage::default()
                        .content(KQT_TEXT.replace("USER", &ctx.author().id.to_string()))
                        .components(also_btn.create(Some(ref_msg.id)))
                        .reference_message(&*ref_msg.clone()),
                )
                .await?;
            return Ok(());
        }
    }

    match ref_msg_arg {
        Some(ref_msg) => {
            ctx.send(CreateReply::default().content("cứ từ từ").ephemeral(true))
                .await?;

            ctx.channel_id()
                .send_message(
                    &ctx.serenity_context(),
                    CreateMessage::default()
                        .content(KQT_TEXT.replace("USER", &ctx.author().id.to_string()))
                        .components(also_btn.create(Some(ref_msg.id)))
                        .reference_message(&ref_msg),
                )
                .await?;
        }
        None => {
            ctx.send(
                CreateReply::default()
                    .content(KQT_TEXT.replace("USER", &ctx.author().id.to_string()))
                    .components(also_btn.create(None)),
            )
            .await?;
        }
    };

    also_btn.handler(ctx).await
}

#[poise::command(context_menu_command = "<user> đã đéo hỏi.")]
pub async fn kqt_cm(ctx: Context<'_>, ref_msg: Message) -> Result<(), Error> {
    let also_btn = super::also_btn::AlsoButton::new(
        KQT_BTN_ID,
        KQT_BTN_LABEL,
        ButtonStyle::Danger,
        KQT_ALSO_TEXT,
    );

    ctx.send(CreateReply::default().content("cứ từ từ").ephemeral(true))
        .await?;

    ctx.channel_id()
        .send_message(
            &ctx.serenity_context(),
            CreateMessage::default()
                .content(KQT_ALSO_TEXT.replace("USER", &ctx.author().id.to_string()))
                .components(also_btn.create(Some(ref_msg.id)))
                .reference_message(&ref_msg),
        )
        .await?;

    Ok(())
}

#[poise::command(context_menu_command = "Ai hỏi bộ trưởng?")]
pub async fn kqt_cm_ai_hoi_bo_truong(ctx: Context<'_>, ref_msg: Message) -> Result<(), Error> {
    ctx.send(CreateReply::default().content("cứ từ từ").ephemeral(true))
        .await?;

    ctx.channel_id()
        .send_message(
            &ctx.serenity_context(),
            CreateMessage::default()
                .content("Ai hỏi mà bộ trưởng trả lời?".to_string())
                .reference_message(&ref_msg),
        )
        .await?;

    Ok(())
}
