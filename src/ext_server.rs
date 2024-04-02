use actix_web::{App, HttpServer};
use actix_web::web::Data;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use dotenv::dotenv;
use std::env;
use influxdb2::Client;
use std::sync::Arc;
use actix_web::middleware::Logger;
use crate::signal_handler::handle_signal;
use crate::db_handler::init_db_client;


pub async fn start_external_signal_server(influxdb_client: Arc<Client>) -> std::io::Result<()> {

    dotenv().ok();

    let ssl_pub = env::var("SSL_PUB").expect("SSL_PUB not found in .env");
    let ssl_prv = env::var("SSL_PRV").expect("SSL_PRV not found in .env");

    println!("Started external signals server...");



    let mut external_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    external_builder
        .set_private_key_file(&ssl_prv, SslFiletype::PEM)
        .unwrap();
    external_builder
        .set_certificate_chain_file(&ssl_pub)
        .unwrap();


//let client_ref = influxdb_client.clone(); 
let client_ref = Data::new(init_db_client().await);


HttpServer::new(move || {
    //println!("Number of strong references: {}", Arc::strong_count(&client_ref));
    //println!("Client type in MAIN config: {}", type_name::<Client>());
    //println!("Client type in MAIN config: {}", type_name::<Arc<Client>>());

    App::new()
        .wrap(Logger::default()) 
        .app_data(client_ref.clone()) 
        .service(handle_signal)
})
    .bind_openssl("0.0.0.0:1025", external_builder)?
    .run()
    .await
}
