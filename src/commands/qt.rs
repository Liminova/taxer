use crate::{Context, Error};

#[poise::command(prefix_command, slash_command, aliases("quantam"))]
pub async fn qt(ctx: Context<'_>) -> Result<(), Error> {
    let reply = format!("{} đã hỏi.", &ctx.author());
    if let Context::Prefix(pctx) = ctx {
        if pctx.msg.referenced_message.is_some() {
            pctx.msg
                .channel_id
                .send_message(&ctx.serenity_context(), |f| {
                    f.content(reply)
                        .reference_message(&*pctx.msg.referenced_message.clone().unwrap())
                })
                .await?;
        } else {
            ctx.send(|f| f.content(reply).reply(true)).await?;
        }
    } else {
        ctx.send(|f| f.content(reply).reply(true)).await?;
    }

    Ok(())
}
