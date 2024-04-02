use influxdb2::models::DataPoint;
use influxdb2::Client;
use std::error::Error;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};
use chrono::Utc;
use std::env;
use dotenv::dotenv;
use futures::stream;
use serde_json::json; 
use crate::signal_handler::SignalData; 

pub async fn init_db_client() -> Client {

    dotenv().ok();

    println!("Initializing InfluxDB client...");

    let host = env::var("INFLUXDB_HOST").expect("INFLUXDB_HOST environment variable not set");
    let org = env::var("INFLUXDB_ORG").expect("INFLUXDB_ORG environment variable not set");
    let token = env::var("INFLUXDB_TOKEN").expect("INFLUXDB_TOKEN environment variable not set");

    let influxdb_client = Client::new(host, org, token);
    println!("InfluxDB connected...");

    influxdb_client
}

/*
pub async fn write_signal_data(influxdb_client: &Client, signal_data: SignalData) {

    let now = Utc::now();
    let point = DataPoint::builder("signals")
        .tag("strat", &signal_data.strat)
        .tag("timescale", &signal_data.timescale)
        .tag("exchange", &signal_data.exchange)
        .tag("alert_type", &signal_data.alert_type)
        .tag("pair", &signal_data.pair)
        .field("amount", signal_data.amount as f64)
        .field("price", signal_data.price as f64)
        .field("current_alert_number", signal_data.current_alert_number as i64)
        .timestamp(now.timestamp_nanos_opt().unwrap_or(0))
        .build()
        .unwrap();

    let influxdb_bucket = "tradex";
    influxdb_client.write(influxdb_bucket, stream::iter(vec![point])).await.unwrap();
    println!("Data written to InfluxDB");
}
*/

pub async fn write_signal_data(influxdb_client: &Client, signal_data: SignalData) -> Result<(), Box<dyn Error>> {
println!("entered write_signal_data");

    let now = Utc::now();
    let point = DataPoint::builder("signals")
        .tag("strat", &signal_data.strat)
        .tag("timescale", &signal_data.timescale)
        .tag("exchange", &signal_data.exchange)
        .tag("alert_type", &signal_data.alert_type)
        .tag("pair", &signal_data.pair)
        .field("amount", signal_data.amount as f64)
        .field("price", signal_data.price as f64)
        .field("current_alert_number", signal_data.current_alert_number as i64)
        .timestamp(now.timestamp_nanos_opt().unwrap_or(0))
        .build()
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    let influxdb_bucket = "tradex";
    influxdb_client
        .write(influxdb_bucket, stream::iter(vec![point]))
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    println!("Data written to InfluxDB");
    Ok(())
}


// KILL DATABASE

pub async fn clear_database(_influxdb_client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let bucket = env::var("INFLUXDB_BUCKET")?;
    let org = env::var("INFLUXDB_ORG")?;
    let token = env::var("INFLUXDB_TOKEN")?;

    let delete_url = format!("http://localhost:8086/api/v2/delete?org={}&bucket={}", org, bucket);

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Token {}", token))?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = reqwest::Client::new();
    let response = client.post(&delete_url)
        .headers(headers)
        .json(&json!({
            "start": "1970-01-01T00:00:00Z",
            "stop": chrono::Utc::now().to_rfc3339(),
            "predicate": r#""_measurement" = "signals""#
        }))
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Failed to clear database: {}", response.text().await?).into())
    }
}