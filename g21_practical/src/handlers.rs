// Import necessary modules and types from external crates and internal modules.
// `actix_web` provides tools for building web servers, including handling HTTP requests and responses.
// `sqlx` is used for database connection pooling. `SqlitePool` is a specific pool type for SQLite.
// `uuid` is used for generating unique identifiers.
// `crate::models` and `crate::auth` denote relative imports from the current project's `models` and `auth` modules, respectively.
use actix_web::{web, HttpResponse, Responder};
use sqlx::SqlitePool;
use uuid::Uuid;

// Models and authentication functionality needed for user, stock, and transaction handling.
use crate::models::{};
use crate::auth;

// Define a function to configure the service, setting up the routes available in this web application.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login_function))) // POST /login 
       .service(web::resource("/projects").route(web::get().to(get_projects))) //GET /projects
       .service(web::resource("/projects").route(web::post().to(add_project))) //POST /projects
        web::scope("/bugs")
            .route("", web::get().to(get_bugs)) // GET /bugs
            .route("/{id}", web::get().to(get_bug_by_id)) // GET /bugs/{id}
            .route("/new", web::post().to(create_bug)) // POST /bugs/new
            .route("/assign", web::post().to(assign_bug)) // POST /bugs/assign
            .route("/{id}", web::patch().to(update_bug_details)) // PATCH /bugs/{id}
            .route("/{id}", web::delete().to(delete_bug)) //delete /bugs/{id}
        
}


// Asynchronous function for user login, expected to receive a JSON payload corresponding to a `User` object.
// Returns a responder, encapsulating an HTTP response.
async fn login(pool: web::Data<SqlitePool>, body: web::Json<User>) -> impl Responder {
    // Simulate login logic. Typically you would verify user credentials against the database.
    if body.username == "admin" {
        // If the username matches "admin", a new token is created using your auth logic.
        let token = auth::create_token(Uuid::new_v4());
        // Respond with a 200 OK status and include the token as JSON.
        HttpResponse::Ok().json(serde_json::json!({ "token": token }))
    } else {
        // If authentication fails, respond with a 401 Unauthorized status.
        HttpResponse::Unauthorized().finish()
    }
}

// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn buy_stock(_pool: web::Data<SqlitePool>, _body: web::Json<Transaction>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}

// Asynchronous function for handling stock sell requests.
// Responds similarly to buy requests.
async fn sell_stock(_pool: web::Data<SqlitePool>, _body: web::Json<Transaction>) -> impl Responder {
    // Respond with a 200 OK status, indicating the sell request was processed.
    HttpResponse::Ok().body("Sell request processed")
}

// Asynchronous function for retrieving user transactions, likely from the database.
// Responds with a list of transactions as JSON.
async fn get_transactions(_pool: web::Data<SqlitePool>) -> impl Responder {
    // For demonstration, respond with a hardcoded list of transactions in JSON format.
    HttpResponse::Ok().json(vec!["tx1", "tx2"])
}