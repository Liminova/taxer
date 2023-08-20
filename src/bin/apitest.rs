use dotenvy::dotenv;
use google_maps::prelude::*;
use std::env;
#[tokio::main]
async fn main() {
    dotenv().expect("failed to load .env file.");
    let google_maps_client = GoogleMapsClient::new(&env::var("API_KEY").unwrap());
    let query = google_maps_client
        .distance_matrix(
            vec![Waypoint::Address(String::from(
                "87 Linh Nam, Hanoi, Vietnam",
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
    let distance = query.rows[0].elements[0].distance.clone().unwrap().text;
    println!("{}", distance);
}
