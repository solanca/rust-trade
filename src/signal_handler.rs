// signal_handler.rs

use actix_web::{web, post, HttpResponse, Responder};

use crate::db_handler::write_signal_data; 
use influxdb2::Client; // 


#[derive(Debug, Clone)]
pub struct SignalData {
    pub strat: String,               // Strategy name or identifier
    pub timescale: String,           // Timescale, e.g., in minutes 1,5,240 etc. 
    pub exchange: String,            // Exchange name, e.g., "binance", "kucoin"
    pub alert_type: String,          // Type of alert, e.g., "Buy", "Sell"
    pub pair: String,                // Trading pair, e.g., "BTC-USDT"
    pub amount: f64,                 // Amount as a floating-point number
    pub price: f64,                  // Price at signal time as a floating-point number
    pub current_alert_number: i32,   // Current alert number as an integer
}

#[derive(Debug)] // Make the error type Debug-printable too
pub enum SignalParsingError {
    InvalidFormat,
    IncorrectNumberOfParts,
}

impl SignalData {
    pub fn from_raw_signal(raw_signal: &str) -> Result<Self, SignalParsingError> {
         println!("Received signal: {}", raw_signal);
        let signal_parts: Vec<&str> = raw_signal.split_whitespace().collect();
        if signal_parts.len() == 8 {
            let amount = signal_parts[5].parse::<f64>().map_err(|_| SignalParsingError::InvalidFormat)?;
            let price = signal_parts[6].parse::<f64>().map_err(|_| SignalParsingError::InvalidFormat)?;
            let current_alert_number = signal_parts[7].parse::<i32>().map_err(|_| SignalParsingError::InvalidFormat)?;

            Ok(Self {
                strat: signal_parts[0].to_string(),
                timescale: signal_parts[1].to_string(),
                exchange: signal_parts[2].to_string(),
                alert_type: signal_parts[3].to_string(),
                pair: signal_parts[4].to_string(),
                amount,
                price,
                current_alert_number,
            })
        } else {
            Err(SignalParsingError::IncorrectNumberOfParts)
        }
    }
}

#[post("/")]
pub async fn handle_signal(body: web::Bytes, influxdb_client: web::Data<Client>) -> impl Responder {

    println!("Incoming request to handle_signal");

    let raw_signal = match String::from_utf8(body.to_vec()) {
        Ok(data) => data,
        Err(e) => {
            println!("Error parsing body: {:?}", e);
            return HttpResponse::BadRequest().finish();
        },
    };

    match SignalData::from_raw_signal(&raw_signal) {
        Ok(signal_data) => {
            println!("Parsed signal data: {:?}", signal_data);
            match write_signal_data(&influxdb_client, signal_data).await {
                Ok(_) => HttpResponse::Ok().body("SUCCESS"),
                Err(e) => {
                    println!("Error writing signal data: {:?}", e);
                    HttpResponse::InternalServerError().body("Error writing data")
                }
            }
        },
        Err(error) => {
            println!("Error parsing signal data: {:?}", error);
            HttpResponse::InternalServerError().body(format!("Error parsing signal: {:?}", error))
        }
    }
}
