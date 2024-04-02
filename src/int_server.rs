use actix_web::error;
use actix_web::{web, get, App, Error, HttpRequest, HttpResponse, HttpServer, Result, Responder, middleware::Logger};
use std::fs;
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use crate::db_handler;
use std::fmt;
use crate::server::MyWebSocket;  // Assuming MyWebSocket is defined in the server module.
use actix_files::NamedFile;
use actix_web_actors::ws;
use std::io::Error as IoError; // Alias for the moment

#[derive(Deserialize)]
pub struct SettingsPayload {
    user_profile: UserProfile,
    account_security: AccountSecurity,
    network_communications: NetworkCommunications,
    exchange_api_keys: ExchangeApiKeys,
    notification_settings: NotificationSettings,
    database_actions: DatabaseActions,
}

#[derive(Deserialize)]
pub struct UserProfile {}
#[derive(Deserialize)]
pub struct AccountSecurity {}
#[derive(Deserialize)]
pub struct NetworkCommunications {}
#[derive(Deserialize)]
pub struct ExchangeApiKeys {}
#[derive(Deserialize)]
pub struct NotificationSettings {}
#[derive(Deserialize)]
pub struct DatabaseActions {}

#[derive(Serialize)] 
struct FileNode {
    name: String,
    children: Option<Vec<FileNode>>, 
}



// quick hardcode 
const directory_path: &str = "public/documents";


// potential file errors

#[derive(Debug)]  // Add 'Debug' for error printing
pub enum FileReadError {
    IoError(std::io::Error), 
    // ... (Add other error cases as needed)
}

impl From<IoError> for FileReadError {
    fn from(err: IoError) -> Self {
        FileReadError::IoError(err)
    }
}


impl std::fmt::Display for FileReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            FileReadError::IoError(err) => write!(f, "IO Error: {}", err),
            // ... (add other cases)
        }
    }
}


// API endpoint functions

pub async fn update_settings(_item: web::Json<SettingsPayload>) -> impl Responder {
    // Update settings logic
    println!("INT_API: Update Settings...");
    HttpResponse::Ok().json("Settings updated")
}

pub async fn test_endpoint() -> impl Responder {
    println!("INT_API: Test!");
    HttpResponse::Ok().body("Endpoint Ping!")
}

pub async fn clear_database_endpoint() -> impl Responder {
    println!("INT_API: KILL DATABASE");
    // potential double init of database !!!!!!!!
    let influxdb_client = db_handler::init_db_client().await; // Initialize the client
//
    match db_handler::clear_database(&influxdb_client).await {
       Ok(_) => HttpResponse::Ok().json("Database cleared"),
       Err(e) => HttpResponse::InternalServerError().body(format!("Error clearing database: {}", e)),
    }
}


fn get_docs(directory_spath: &str, relative_path: &str) -> Result<FileNode, FileReadError> {
    println!("INT_API: Get Docs Function...");
    println!("directory_path: {}, relative_path: {}", directory_spath, relative_path);

    // organise directories into treebeard structure 

    let mut root_node = FileNode {
        
        name: relative_path.strip_prefix("public/documents/").unwrap_or(relative_path).to_string(), // Change here
        children: Some(Vec::new()), 
    };


println!("Root node directory_path: {}, relative_path: {}", directory_spath, relative_path);


    for entry in fs::read_dir(directory_spath)? {
        println!("reading through directory_path: {}, relative_path: {}", directory_spath, relative_path);
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

        let mut new_relative_path = relative_path.to_string(); 
        new_relative_path.push_str("/"); 
        new_relative_path.push_str(&file_name);

        if path.is_dir() {
            println!("get_docs: DIR: directory_path: {}, relative_path: {}", directory_spath, relative_path);
            // Recurse with updated relative path
            let child_node = get_docs(path.to_str().unwrap(), &new_relative_path)?;
            root_node.children.as_mut().unwrap().push(child_node);    
        } else {
            // Add files with the relative path as well
            println!("get_docs: FILE: directory_path: {}, relative_path: {}", directory_spath, relative_path);
            let file_node = FileNode {
                name: new_relative_path, 
                children: None,  
            };
            root_node.children.as_mut().unwrap().push(file_node);
        }
    }
println!("Root node directory_path: {}, relative_path: {}", directory_spath, relative_path);
    Ok(root_node)
}





// Endpoint handler using get_docs
async fn get_docs_endpoint() -> HttpResponse {
     println!("INT_API: Get Docs Endpoint...");
   let file_data = match get_docs(directory_path, directory_path) { 
       Ok(data) => data, 
       Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
   };

   HttpResponse::Ok().json(file_data) 
} 



//#[get("/get_file/{filepath}")]
async fn get_file_endpoint(filepath: web::Path<String>) -> Result<impl Responder> {
    println!("GET FILE PATH: {}", filepath);
    let base_path = "public/documents";
    let full_path = format!("{}/{}", base_path, filepath);

    let contents = fs::read_to_string(full_path)
        .map_err(|_| error::ErrorNotFound("File not found"))?;

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(contents))
}



#[derive(Serialize)] 
struct FileInfo {
    name: String,
    // size: u64, 
    // ... other metadata if needed
} 

    // define docs directory



pub fn config(cfg: &mut web::ServiceConfig) {
    
     // Define CORS for the /api scope
    let cors_api = Cors::permissive();

     cfg.service(
        web::scope("/api")
            .wrap(cors_api)  // Use CORS middleware for the API scope
            .route("/test", web::get().to(test_endpoint))
            .route("/update_settings", web::post().to(update_settings))
            .route("/clear_database", web::post().to(clear_database_endpoint))
            .route("/get_docs", web::get().to(get_docs_endpoint))
            // Define the route for get_file_endpoint manually
            .route("/get_file/{filepath:.*}", web::get().to(get_file_endpoint)),
    );
    // Define CORS for the /ws route
    let cors_ws = Cors::permissive();

    cfg.service(
        web::resource("/ws")
            .wrap(cors_ws)  // Use CORS middleware for the WebSocket route
            .route(web::get().to(echo_ws)),
    );
}


pub async fn start_int_server() -> std::io::Result<()> {

     println!("Started internal API and Websocket server...");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default()) // Optional: Add logging middleware
            .configure(config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}




// ------------------------
async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

/// WebSocket handshake and start `MyWebSocket` actor.
async fn echo_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(MyWebSocket::new(), &req, stream)

}