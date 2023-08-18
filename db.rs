use sqlite::Connection;

fn main() {
    let connection = Connection::open("food.db").unwrap();
    let check = "
    CREATE TABLE IF NOT EXISTS db(
        Address TEXT NOT NULL,
        Name TEXT NOT NULL,
        Distance_from_LB REAL NOT NULL,
        Distance_from_HUST REAL NOT NULL
    )
    ";
    connection.execute(check, []).unwrap();
    println!("Table checked.");
}
