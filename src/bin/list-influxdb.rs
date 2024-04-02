use chrono::Utc;
use influxdb2::models::Query;
use influxdb2::FromDataPoint;
use std::env;
use dotenv::dotenv;
use chrono::{DateTime, FixedOffset};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    dotenv().ok();

    // Define the InfluxDB connection parameters
    let host = env::var("INFLUXDB_HOST").expect("Error: INFLUXDB_HOST environment variable not set");
    let org = env::var("INFLUXDB_ORG").expect("Error: INFLUXDB_ORG environment variable not set");
    let token = env::var("INFLUXDB_TOKEN").expect("Error: INFLUXDB_TOKEN environment variable not set");
    let bucket = "tradex"; // Replace with your actual bucket name

    // Initialize the InfluxDB client
    let client = influxdb2::Client::new(host, org, token);

    // Define a struct for data modeling
    #[derive(FromDataPoint)]
    struct Measurement {
        strat: String,
        timescale: String,
        exchange: String,
        alert_type: String,
        pair: String,
        amount: String,
        price: String,
        current_alert_number: String,
    }

impl Default for Measurement {
    fn default() -> Self {
        Self { 
            strat: "default_strategy".to_string(),
            timescale: "1m".to_string(), // Adjust as needed
            exchange: "default_exchange".to_string(),
            alert_type: "default_alert".to_string(),
            pair: "".to_string(), // Empty if no default
            amount: "0".to_string(), // Assuming amounts are strings
            price: "0".to_string(), // Assuming prices are strings
            current_alert_number: "0".to_string(), 
         }
    }
}

    // Flux Query to retrieve all records from "signals"
    let query_string = format!("from(bucket: \"{}\") 
                                |> range(start: -7d) 
                                |> filter(fn: (r) => r._measurement == \"signals\")", bucket);

    // Construct the query
    let query = Query::new(query_string);

    // Execute the query 
    let results: Vec<Measurement> = client.query::<Measurement>(Some(query))
                                          .await?;


// Organize results by timescale and strat
let mut organized_results: HashMap<(String, String), Vec<Measurement>> = HashMap::new();

for record in results {
    let key = (record.timescale.clone(), record.strat.clone()); // Create a unique key
    organized_results.entry(key).or_insert_with(Vec::new).push(record);
}

// Display (or further process) the organized data
for ((timescale, strat), records) in organized_results {
    println!("--- Timescale: {}, Strat: {} ---", timescale, strat);
 for record in records {
       println!("Price: {}", record.price);
    println!("Exchange: {} Alert Type: {} Pair: {} Amount: {} Price: {} Current Alert Number: {}", 
             record.exchange, record.alert_type, record.pair, record.amount, 
             record.price, record.current_alert_number);
    println!(); // Additional line break between records
}
}

    Ok(())
}
