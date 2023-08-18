use crate::{Context, Error};
use sqlite::Connection;

// Parameters: t!food <location_from> <desired_distance>

#[poise::command(prefix_command, slash_command, aliases("food"))]
pub async fn food(ctx: Context<'_>, location: String, max_distance: String) -> Result<(), Error> {
    if location != "LB" || location != "HUST" {
        ctx.send(|f| f.content("Invalid location.")).await?;
    }

    ctx.defer().await?;

    let connection = Connection::open("food.db").unwrap();
    let mut list = Vec::new();
    for row in connection.prepare("SELECT * FROM db WHERE Distance_from_? < ?") {
        list.push(row.unwrap());
    }
    let random = rand::thread_rng().gen_range(0..list.len());
    ctx.send(|f| f.content(list[random])).await?;
    Ok(())
}
