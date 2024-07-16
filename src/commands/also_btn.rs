use crate::{Context, Error};

use poise::serenity_prelude::{
    ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, MessageId,
};

pub struct AlsoButtonCreator {
    btn_id: String,
    btn_label: String,
    btn_style: ButtonStyle,
    also_msg: String,
}

impl AlsoButtonCreator {
    pub fn new(btn_id: &str, btn_label: &str, btn_style: ButtonStyle, also_msg: &str) -> Self {
        AlsoButtonCreator {
            btn_id: btn_id.to_string(),
            btn_label: btn_label.to_string(),
            btn_style,
            also_msg: also_msg.to_string(),
        }
    }

    pub fn create(&self, ref_msg_id: Option<MessageId>) -> Vec<CreateActionRow> {
        vec![CreateActionRow::Buttons(vec![CreateButton::new(
            match ref_msg_id {
                Some(ref_msg_id) => format!("{}{}", self.btn_id, ref_msg_id),
                None => self.btn_id.to_string(),
            },
        )
        .label(self.btn_label.clone())
        .style(self.btn_style)])]
    }

    pub async fn handler(&self, ctx: Context<'_>) -> Result<(), Error> {
        while let Some(mci) = ComponentInteractionCollector::new(ctx.serenity_context())
            .timeout(std::time::Duration::from_secs(300))
            .filter({
                let btn_id = self.btn_id.clone();
                move |i| i.data.custom_id.starts_with(&btn_id)
            })
            .await
        {
            match mci
                .data
                .custom_id
                .strip_prefix(&self.btn_id)
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

                    mci.create_response(
                        &ctx.serenity_context(),
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::default()
                                .content("cứ từ từ")
                                .ephemeral(true),
                        ),
                    )
                    .await?;

                    ctx.channel_id()
                        .send_message(
                            &ctx.serenity_context(),
                            CreateMessage::default()
                                .content(&self.also_msg.replace("USER", &mci.user.id.to_string()))
                                .components(self.create(Some(ref_msg.id)))
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
                                .content(&self.also_msg.replace("USER", &mci.user.id.to_string()))
                                .components(self.create(Some(mci.message.id)))
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
