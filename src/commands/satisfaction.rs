use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    prelude::{Context, SerenityError},
};
use rand::Rng;

pub fn out_of_ten() -> i32 {
    let mut rng = rand::thread_rng();
    let num = rng.gen_range(-1..11);
    num
}


pub async fn run(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
) -> Result<(), SerenityError> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content(format!("Your satisfaction level is: **{}/10**", out_of_ten()))
                })
        }).await
}
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("satisfaction")
        .description("A command that measures your current satisfaction level.")
}