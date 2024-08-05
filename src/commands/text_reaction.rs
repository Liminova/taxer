use crate::{AppError, Context};

use anyhow::anyhow;
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton,
        CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, MessageId,
    },
    CreateReply,
};
use tracing::warn;

pub struct TextReactionBuilder {
    label: String,
    also_label: String,
    custom_id: String,
    button_style: ButtonStyle,
    button_label: String,
    ref_msg_id: Option<String>,
}

impl Default for TextReactionBuilder {
    fn default() -> Self {
        TextReactionBuilder {
            label: "".to_string(),
            also_label: "".to_string(),
            custom_id: "".to_string(),
            button_label: "".to_string(),
            button_style: ButtonStyle::Primary,
            ref_msg_id: None,
        }
    }
}

impl TextReactionBuilder {
    pub fn set_label(mut self, label: impl ToString) -> Self {
        self.label = label.to_string();
        self
    }
    pub fn set_also_label(mut self, also_label: impl ToString) -> Self {
        self.also_label = also_label.to_string();
        self
    }
    pub fn set_custom_id(mut self, custom_id: impl ToString) -> Self {
        self.custom_id = custom_id.to_string();
        self
    }
    pub fn set_button_label(mut self, button_label: impl ToString) -> Self {
        self.button_label = button_label.to_string();
        self
    }
    pub fn set_button_style(mut self, button_style: ButtonStyle) -> Self {
        self.button_style = button_style;
        self
    }
    pub fn set_ref_msg_id(mut self, ref_msg_id: Option<impl ToString>) -> Self {
        self.ref_msg_id = ref_msg_id.map(|s| s.to_string());
        self
    }
    pub fn build(self) -> TextReaction {
        TextReaction {
            label: self.label,
            also_label: self.also_label,
            custom_id: self.custom_id,
            button_label: self.button_label,
            button_style: self.button_style,
            ref_msg_id: self.ref_msg_id,
        }
    }
}

pub struct TextReaction {
    label: String,
    also_label: String,
    custom_id: String,
    button_label: String,
    button_style: ButtonStyle,
    ref_msg_id: Option<String>,
}

impl TextReaction {
    pub async fn slash_command(&self, ctx: &Context<'_>) -> Result<(), AppError> {
        if let Context::Prefix(pctx) = ctx {
            if let Some(ref_msg) = pctx.msg.referenced_message.as_ref() {
                pctx.msg
                    .channel_id
                    .send_message(
                        &ctx.serenity_context(),
                        CreateMessage::default()
                            .content(
                                self.also_label
                                    .replace("USER", &ctx.author().id.to_string()),
                            )
                            .components(self.create_components(Some(ref_msg.id)))
                            .reference_message(&*ref_msg.clone()),
                    )
                    .await?;
                return Ok(());
            }
        }

        let ref_msg =
            if let Some(ref_msg) = self.ref_msg_id.as_ref().and_then(|s| s.parse::<u64>().ok()) {
                match ctx
                    .channel_id()
                    .message(&ctx.serenity_context(), ref_msg)
                    .await
                {
                    Ok(ref_msg) => Some(ref_msg),
                    Err(_) => None,
                }
            } else {
                None
            };

        if let Some(ref_msg) = ref_msg {
            match ctx
                .send(CreateReply::default().content("cứ từ từ").ephemeral(true))
                .await
            {
                Ok(reply_handler) => {
                    if let Err(e) = reply_handler.delete(*ctx).await {
                        return Err(AppError::from(anyhow!("can't delete 'defer' msg: {}", e)));
                    };
                }
                Err(e) => {
                    return Err(AppError::from(anyhow!("can't reply 'defer' msg: {}", e)));
                }
            }

            ctx.channel_id()
                .send_message(
                    ctx.serenity_context(),
                    CreateMessage::default()
                        .content(self.label.replace("USER", &ctx.author().id.to_string()))
                        .components(self.create_components(Some(ref_msg.id)))
                        .reference_message(&ref_msg),
                )
                .await?;
        } else {
            ctx.send(
                CreateReply::default()
                    .content(self.label.replace("USER", &ctx.author().id.to_string()))
                    .components(self.create_components(None)),
            )
            .await
            .map_err(|e| AppError::from(anyhow!("can't reply non-ref msg: {}", e)))?;
        }
        self.handler(ctx).await
    }

    pub async fn context_menu_command(
        &self,
        ctx: &Context<'_>,
        ref_msg: &poise::serenity_prelude::Message,
    ) -> Result<(), AppError> {
        match ctx
            .send(CreateReply::default().content("cứ từ từ").ephemeral(true))
            .await
        {
            Ok(reply_handler) => {
                if let Err(e) = reply_handler.delete(*ctx).await {
                    return Err(AppError::from(anyhow!("can't delete 'defer' msg: {}", e)));
                };
            }
            Err(e) => {
                return Err(AppError::from(anyhow!("can't reply 'defer' msg: {}", e)));
            }
        }

        ctx.channel_id()
            .send_message(
                ctx.serenity_context(),
                CreateMessage::default()
                    .content(self.label.replace("USER", &ctx.author().id.to_string()))
                    .components(self.create_components(Some(ref_msg.id)))
                    .reference_message(ref_msg),
            )
            .await?;

        self.handler(ctx).await
    }

    fn create_components(&self, ref_msg_id: Option<MessageId>) -> Vec<CreateActionRow> {
        vec![CreateActionRow::Buttons(vec![CreateButton::new(
            match ref_msg_id {
                Some(ref_msg_id) => format!("{}{}", self.custom_id, ref_msg_id),
                None => self.custom_id.to_string(),
            },
        )
        .label(self.button_label.clone())
        .style(self.button_style)])]
    }

    /// Handlers for the button "also" message
    async fn handler(&self, ctx: &Context<'_>) -> Result<(), AppError> {
        while let Some(mci) = ComponentInteractionCollector::new(ctx.serenity_context())
            .timeout(std::time::Duration::from_secs(300))
            .filter({
                let btn_id = self.custom_id.clone();
                move |i| i.data.custom_id.starts_with(&btn_id)
            })
            .await
        {
            match mci
                .data
                .custom_id
                .strip_prefix(&self.custom_id)
                .filter(|s| !s.is_empty())
                .and_then(|s| s.parse::<u64>().ok())
                .filter(|&s| s != 0)
            {
                // if exists ref msg id in button's custom id
                Some(ref_msg_id) => {
                    let ref_msg = ctx
                        .channel_id()
                        .message(&ctx.serenity_context(), ref_msg_id)
                        .await?;

                    match mci
                        .create_response(
                            &ctx.serenity_context(),
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::default()
                                    .content("cứ từ từ")
                                    .ephemeral(true),
                            ),
                        )
                        .await
                    {
                        Ok(_) => {
                            mci.delete_response(&ctx.serenity_context()).await?;
                        }

                        Err(e) => warn!("can't reply 'defer' msg: {}", e),
                    };

                    ctx.channel_id()
                        .send_message(
                            &ctx.serenity_context(),
                            CreateMessage::default()
                                .content(self.also_label.replace("USER", &mci.user.id.to_string()))
                                .components(self.create_components(Some(ref_msg.id)))
                                .reference_message(&ref_msg),
                        )
                        .await?;
                }
                // else reply the user's command
                None => {
                    mci.create_response(
                        &ctx.serenity_context(),
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::default()
                                .content(self.also_label.replace("USER", &mci.user.id.to_string()))
                                .components(self.create_components(Some(mci.message.id)))
                                .ephemeral(false),
                        ),
                    )
                    .await?;
                }
            };
        }

        Ok(())
    }
}
