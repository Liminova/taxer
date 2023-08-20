use crate::{Context, Error};
use rand::prelude::*;
use sqlite::{Connection, State};

// Parameters: t!food <location_from> <desired_distance>

#[poise::command(prefix_command, slash_command, aliases("food"))]
pub async fn food(ctx: Context<'_>, location: String, max_distance: String) -> Result<(), Error> {
    if location != "LB" || location != "HUST" {
        ctx.send(|f| f.content("Invalid location.")).await?;
        return Ok(());
    }

    ctx.defer().await?;

    let connection = Connection::open("food.db").unwrap();
    let mut list = Vec::new();
    let query = format!(
        "SELECT Name FROM db WHERE Distance_from_{} <= {}",
        location, max_distance
    );
    // let mut statement = connection.prepare(query).unwrap();
    // while let Ok(State::Row) = statement.next() {
    //     let row = statement.read::<String, _>("name").unwrap();
    //     list.push(row);
    // }
    while let Ok(State::Row) = connection.prepare(query.clone()).unwrap().next() {
        let row = connection
            .prepare(query.clone())
            .unwrap()
            .read::<String, _>("name")
            .unwrap();
        list.push(row);
    }
    let random = rand::thread_rng().gen_range(0..list.len());
    let name = list[random].clone();
    let place = connection
        .prepare(format!("SELECT Address FROM db WHERE Name = {}", name))
        .unwrap()
        .read::<String, _>("Address")
        .unwrap();
    let distance_from_place = connection
        .prepare(format!(
            "SELECT Distance_from_{} FROM db WHERE Name = {}",
            location, name
        ))
        .unwrap()
        .read::<f64, _>("Distance")
        .unwrap();
    ctx.send(|f| {
        f.content(format!(
            "You should go to {} at {} ({} km from {})",
            name, place, distance_from_place, location
        ))
    })
    .await?;
    Ok(())
}
