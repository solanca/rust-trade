mod signal_handler;
//mod strategy_engine;
mod db_handler;
mod ext_server;
mod int_server; 
mod server;
mod client;
use std::sync::Arc;

// app functions

//use crate::strategy_engine::start_strategy_engine;
use db_handler::init_db_client;


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    env_logger::init(); // Initialize logging

    // Start the barter-rs data stream
    //barter_interface::start_data_stream().await; 
    //println!("Started barter-rs...");

    println!("Starting Actix servers...");

    // Start the strategy engine
    //start_strategy_engine();

    // Initialize the InfluxDB client
    let influxdb_client = Arc::new(init_db_client().await); 

  // Define the external signals server
    let ext_server_handle = actix_rt::spawn(async move {
        ext_server::start_external_signal_server(influxdb_client.clone()).await.expect("External server failed");
    });

    // Define the internal API server
    let int_server_handle = actix_rt::spawn(async move {
        int_server::start_int_server().await.expect("Internal server failed");
    });

 // Await both servers to finish
    let _ = tokio::join!(ext_server_handle, int_server_handle);




    Ok(())
}