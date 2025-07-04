// Import necessary modules and types from external crates and internal modules.
// `actix_web` provides tools for building web servers, including handling HTTP requests and responses.
// `sqlx` is used for database connection pooling. `SqlitePool` is a specific pool type for SQLite.
// `uuid` is used for generating unique identifiers.
// `crate::models` and `crate::auth` denote relative imports from the current project's `models` and `auth` modules, respectively.
use actix_web::{web, HttpResponse, Responder};
use sqlx::SqlitePool;
use uuid::Uuid;
use tera::{Tera, Context};

// Models and authentication functionality needed for user, stock, and transaction handling.
use crate::models::{User, BugReport, LoginRequest, LoginResponse, CreateBug, ProjectRecord, BugAssignment, SimpleUser};
use crate::auth;

// Define a function to configure the service, setting up the routes available in this web application.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login_function))) // POST /login 
       .service(web::resource("/projects").route(web::get().to(get_projects))) //GET /projects
       .service(web::resource("/projects").route(web::post().to(add_project))) //POST /projects
       .service(
            web::scope("/bugs")
                .route("", web::get().to(get_bugs)) // GET /bugs
                .route("/assign", web::get().to(render_bug_form)) // GET /bugs/assign
                .route("/assign", web::post().to(assign_bug)) // POST /bugs/assign
                .route("/{id}", web::get().to(get_bug_by_id)) // GET /bugs/{id}
                .route("/new", web::post().to(create_bug)) // POST /bugs/new
                .route("/{id}", web::patch().to(update_bug_details)) // PATCH /bugs/{id}
                .route("/{id}", web::delete().to(delete_bug)) //delete /bugs/{id}
       );
}

// Asynchronous function for user login, expected to receive a JSON payload corresponding to a `User` object.
// Returns a responder, encapsulating an HTTP response.
async fn login_function(
    pool: web::Data<SqlitePool>, 
    body: web::Json<LoginRequest>
) -> impl Responder {
    let salt = "bugtrack2025";
    let user = sqlx::query_as::<_, User>(
             "SELECT id, username, hashed_password FROM users WHERE username = ?",
        )
        .bind(&body.username)
        .fetch_optional(pool.get_ref())
        .await;

    match user {
        Ok(Some(user)) => {
            // Verify password
            match auth::verify_with_salt(&body.password, salt, &user.hashed_password) {
                Ok(true) => {
                    // Password correct, create token
                    let token = auth::create_token(Uuid::new_v4());

                    HttpResponse::Ok().json(LoginResponse {
                        status: "success".to_string(),
                        message: "Login successful".to_string(),
                        token: Some(token),
                    })
                }
                Ok(false) => {
                    // Password incorrect
                    HttpResponse::Unauthorized().json(LoginResponse {
                        status: "failure".to_string(),
                        message: "Invalid credentials".to_string(),
                        token: None,
                    })
                }
                Err(_) => {
                    // Error verifying password
                    HttpResponse::InternalServerError().json(LoginResponse {
                        status: "failure".to_string(),
                        message: "Internal server error".to_string(),
                        token: None,
                    })
                }  
            }
        }
        Ok(None) => {
            // User not found
            HttpResponse::Unauthorized().json(LoginResponse {
                status: "failure".to_string(),
                message: "Invalid credentials".to_string(),
                token: None,
            })
        }
        Err(_) => {
            // Database error
            HttpResponse::InternalServerError().json(LoginResponse {
                status: "failure".to_string(),
                message: "Database error".to_string(),
                token: None,
            })
        }
    }
}

// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn get_projects(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn add_project(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn get_bugs(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn get_bug_by_id(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}


// Asynchronous function for creating a new bug report.
// Simply responds to the request with a confirmation message.
async fn create_bug(_pool: web::Data<SqlitePool>, _body: web::Json<CreateBug>) -> impl Responder {
    
    let user = match sqlx::query_as::<_, User>(
        "SELECT id, username, hashed_password FROM users WHERE username = ?"
    )
    .bind(&_body.reported_by)
    .fetch_optional(_pool.get_ref())
    .await {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::NotFound().body("User not found"),
        Err(e) => {
            eprintln!("User query error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let project = match sqlx::query_as::<_, ProjectRecord>(
        "SELECT id, project_name, project_description, created_at, user_id  FROM projectRecord WHERE project_name = ?"
    )
    .bind(&_body.project_name)
    .fetch_optional(_pool.get_ref())
    .await {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::NotFound().body("Project not found"),
        Err(e) => {
            eprintln!("Project query error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let bug_id = uuid::Uuid::new_v4();
    
    if let Err(e) = sqlx::query("INSERT INTO bugReport (id, project_id, title, description, reported_by, severity, is_fixed) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&bug_id.as_bytes()[..])
        .bind(&project.id)// project_id from session
        .bind(&_body.title)
        .bind(&_body.description)
        .bind(&user.id) // user_id from session
        .bind(&_body.severity)
        .bind(false)
        .execute(_pool.get_ref())
        .await
    {
        eprintln!("BugReport insert error: {:?}", e);
        return HttpResponse::InternalServerError().finish();
    }

    let response = BugReport {
        id: bug_id,
        project_id: project.id,
        title: _body.title.clone(),
        description: _body.description.clone(),
        reported_by: user.id,
        severity: _body.severity.clone(),
        fixed_by: None, // Initially set to nil, as the bug is not fixed yet
        created_at: chrono::Utc::now().to_rfc3339(), // Current timestamp in RFC 3339 format
        is_fixed: false,
    };

    HttpResponse::Ok().json(response)
}


// Asynchronous function to render the bug assignment form.
async fn render_bug_form(pool: web::Data<SqlitePool>) -> impl Responder {
    println!("render_bug_form called");
    
    // Fetch open bugs
    let open_bugs = sqlx::query_as::<_, BugReport>(
        "SELECT * FROM bugReport WHERE is_fixed = false"
    )
    .fetch_all(pool.get_ref())
    .await;

    // Fetch all users
    let users = sqlx::query_as::<_, SimpleUser>(
        "SELECT id, username FROM users"
    )
    .fetch_all(pool.get_ref())
    .await;

    println!("Bugs result: {:?}", open_bugs.is_ok());
    println!("Users result: {:?}", users.is_ok());

    match (open_bugs, users) {
        (Ok(bugs), Ok(user_list)) => {
            println!("Found {} bugs and {} users", bugs.len(), user_list.len());
            
            // Create Tera instance
            let tera = match Tera::new("static/*.html") {
                Ok(t) => {
                    println!("Tera instance created successfully");
                    t
                },
                Err(e) => {
                    eprintln!("Tera parsing error: {}", e);
                    return HttpResponse::InternalServerError().body("Template parsing error");
                }
            };

            let mut context = Context::new();
            context.insert("bugs", &bugs);
            context.insert("users", &user_list);

            // Render the template
            match tera.render("bugform.html", &context) {
                Ok(rendered) => {
                    println!("Template rendered successfully");
                    HttpResponse::Ok().content_type("text/html").body(rendered)
                },
                Err(e) => {
                    eprintln!("Template rendering error: {}", e);
                    HttpResponse::InternalServerError().body("Failed to render template")
                }
            }
        }
        (Err(e), _) => {
            eprintln!("Error fetching bugs from database: {:?}", e);
            HttpResponse::InternalServerError().body("Database error fetching bugs")
        }
        (_, Err(e)) => {
            eprintln!("Error fetching users from database: {:?}", e);
            HttpResponse::InternalServerError().body("Database error fetching users")
        }
    }
}

async fn assign_bug(pool: web::Data<SqlitePool>, body: web::Json<BugAssignment>) -> impl Responder {
    let result = sqlx::query(
        "UPDATE bugReport SET fixed_by = ? WHERE id = ?"
    )
    .bind(&body.user_id.as_bytes()[..])
    .bind(&body.bug_id.as_bytes()[..])
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Bug assigned successfully"),
        Err(e) => {
            eprintln!("Bug assignment error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to assign bug")
        }
    }
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn update_bug_details(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn delete_bug(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}