use crate::{Context, Error};
use dotenv::dotenv;
use dotenvy::dotenv;
use google_maps::prelude::*;
use sqlite::Connection;
use std::env;

// Parameters: t!add <address> <name>
#[poise::command(prefix_command, slash_command, aliases("add"))]
pub async fn add(ctx: Context<'_>, location: String, name: String) -> Result<(), Error> {
    dotenv().expect("failed to load .env file.");
    let google_maps_client = GoogleMapsClient::new(&env::var("API_KEY").unwrap());
    let connection = Connection::open("food.db").unwrap();

    ctx.defer().await?;

    let query = google_maps_client
        .distance_matrix(
            vec![Waypoint::Address(String::from(
                format! {"{}, Hanoi, Vietnam", location},
            ))],
            vec![
                Waypoint::Address(String::from("Hanoi University of Science and Technology")),
                Waypoint::Address(String::from("AEON MALL Long Bien")),
            ],
        )
        .with_unit_system(UnitSystem::Metric)
        .execute()
        .await
        .unwrap();
    let distance_from_hust = query.rows[0].elements[0]
        .distance
        .clone()
        .unwrap()
        .text
        .replace(" km", "")
        .parse::<f64>()
        .unwrap();
    let distance_from_lb = query.rows[0].elements[1]
        .distance
        .clone()
        .unwrap()
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
