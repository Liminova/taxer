use crate::{Context, Error};
use dotenv::dotenv;
use google_maps::prelude::*;

// Parameters: t!add <address> <name>
#[poise::command(prefix_command, slash_command, aliases("add"))]
pub async fn add(ctx: Context<'_>, location: String, name: String) -> Result<(), Error> {
    let google_maps_client = GoogleMapsClient::new("API_KEY");
    let connection = Connection::open("food.db").unwrap();

    ctx.defer().await?;

    let query_from_H = google_maps_client
        .distance_matrix(
            vec![Waypoint::Address(String::from(
                format! {"{}, Hanoi", location},
            ))],
            vec![Waypoint::Address(String::from(
                "Hanoi University of Science and Technology",
            ))],
        )
        .with_unit_system(UnitSystem::Metric)
        .execute()
        .await?;
    let query_from_LB = google_maps_client
        .distance_matrix(
            vec![Waypoint::Address(String::from(
                format! {"{}, Hanoi", location},
            ))],
            vec![Waypoint::Address(String::from("AEON MALL Long Bien"))],
        )
        .with_unit_system(UnitSystem::Metric)
        .execute()
        .await?;
    let distance_from_H = query_from_H.rows[0].elements[0]
        .distance
        .text
        .replace(" km", "")
        .parse::<f64>()
        .unwrap();
    let distance_from_LB = query_from_LB.rows[0].elements[0]
        .distance
        .text
        .replace(" km", "")
        .parse::<f64>()
        .unwrap();
    let insert = format!(
        "INSERT INTO db VALUES ({}, {}, {}, {})",
        location, name, distance_from_LB, distance_from_H
    );
    connection.execute(insert, []).unwrap();
    ctx.send(|f| f.content("Added successfully!")).await?;
}
