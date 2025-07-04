// Import necessary modules and types from external crates and internal modules.
// `actix_web` provides tools for building web servers, including handling HTTP requests and responses.
// `sqlx` is used for database connection pooling. `SqlitePool` is a specific pool type for SQLite.
// `uuid` is used for generating unique identifiers.
// `crate::models` and `crate::auth` denote relative imports from the current project's `models` and `auth` modules, respectively.
use actix_web::{web, HttpResponse, Responder};
use sqlx::SqlitePool;
use uuid::Uuid;
use hex;

// Models and authentication functionality needed for user, stock, and transaction handling.
use crate::models::{User, BugReport, LoginRequest, LoginResponse, CreateBug, ProjectRecord, BugFilter, UpdateBugReport};
use crate::auth;

// Define a function to configure the service, setting up the routes available in this web application.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login_function))) // POST /login 
       .service(web::resource("/projects").route(web::get().to(get_projects))) //GET /projects
       .service(web::resource("/projects").route(web::post().to(add_project))) //POST /projects
       .service(
            web::scope("/bugs")
                .route("", web::get().to(get_bugs)) // GET /bugs
                .route("/{id}", web::get().to(get_bug_by_id)) // GET /bugs/{id}
                .route("/new", web::post().to(create_bug)) // POST /bugs/new
                .route("/assign", web::post().to(assign_bug)) // POST /bugs/assign
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
async fn get_projects(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}


// Asynchronous function for handling stock purchase requests.
async fn add_project(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}

// Asynchronous function for fetching bug reports based on optional filters.
async fn get_bugs(_pool: web::Data<SqlitePool>, _filter: web::Query<BugFilter>) -> impl Responder {
    let mut query = "SELECT id, project_id, title, description, reported_by, fixed_by, severity, is_fixed, created_at FROM bugReport WHERE 1=1"
        .to_string();

    if let Some(is_fixed) = _filter.is_fixed {
        query.push_str(&format!(" AND is_fixed = {}", if is_fixed { 1 } else { 0 }));
    }

    if let Some(severity) = &_filter.severity {
        query.push_str(&format!(" AND severity = '{}'", severity.replace('\'', "''"))); 
    }

    if let Some(project_name) = &_filter.project_name {
        // Query for project record by name
        let project = match sqlx::query_as::<_, ProjectRecord>(
            "SELECT id, project_name, project_description, created_at, user_id FROM projectRecord WHERE project_name = ?"
        )
        .bind(project_name)
        .fetch_optional(_pool.get_ref())
        .await {
            Ok(Some(project)) => project,
            Ok(None) => return HttpResponse::NotFound().body("Project not found"),
            Err(e) => {
                eprintln!("Error fetching project: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

        // Now add the project_id condition to the query
        let project_id_bytes = project.id.as_bytes().to_vec();
        query.push_str(&format!(" AND project_id = x'{}'", hex::encode(project_id_bytes)));
    }

    // Execute the final query
    match sqlx::query_as::<_, BugReport>(&query).fetch_all(_pool.get_ref()).await {
        Ok(bugs) => HttpResponse::Ok().json(bugs),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to fetch bugs")
        }
    }
}

// Asynchronous function for getting a bug by its ID in it path.
async fn get_bug_by_id(_pool: web::Data<SqlitePool>, _bug_id: web::Path<Uuid>) -> impl Responder {
    match sqlx::query_as::<_, BugReport>(
        "SELECT id, project_id, title, description, reported_by, fixed_by, severity, is_fixed, created_at \
         FROM bugReport WHERE id = ?"
    )
    .bind(_bug_id.into_inner())
    .fetch_optional(_pool.get_ref())
    .await {
        Ok(Some(bug)) => HttpResponse::Ok().json(bug),
        Ok(None) => HttpResponse::NotFound().body("Bug not found"),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to fetch bug")
        }
    }   
}


// Asynchronous function for creating a new bug report.
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
        fixed_by: Some(uuid::Uuid::nil()), // Initially set to nil, as the bug is not fixed yet
        created_at: chrono::Utc::now().to_rfc3339(), // Current timestamp in RFC 3339 format
        is_fixed: false,
    };

    HttpResponse::Ok().json(response)
}


// Asynchronous function for handling stock purchase requests.
async fn assign_bug(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    HttpResponse::Ok().body("Buy request processed")
}


// Asynchronous function to update a bug details.
pub async fn update_bug_details(_pool: web::Data<SqlitePool>,_bug_id: web::Path<Uuid>,_body: web::Json<UpdateBugReport>) -> impl Responder {

    let mut set_clauses = Vec::new();
    let mut string_params = Vec::new();
    let mut blob_params = Vec::new();

    // Handle each optional field
    if let Some(is_fixed) = _body.is_fixed {
        set_clauses.push("is_fixed = ?");
        string_params.push(if is_fixed { "1" } else { "0" });
    }

    if let Some(severity) = &_body.severity {
        set_clauses.push("severity = ?");
        string_params.push(severity.as_str());
    }

    if let Some(description) = &_body.description {
        set_clauses.push("description = ?");
        string_params.push(description.as_str());
    }

    if let Some(fixed_by_username) = &_body.fixed_by {
        // Fetch user by username
        let user = match sqlx::query_as::<_, User>(
            "SELECT id, username, hashed_password FROM users WHERE username = ?",
        )
        .bind(fixed_by_username)
        .fetch_optional(_pool.get_ref())
        .await {
            Ok(Some(user)) => user,
            Ok(None) => return HttpResponse::NotFound().body("User not found"),
            Err(e) => {
                eprintln!("Database error: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

        set_clauses.push("fixed_by = ?");
        blob_params.push(user.id.as_bytes().to_vec());
    }

    if set_clauses.is_empty() {
        return HttpResponse::BadRequest().body("No fields provided for update");
    }

    // Build final query
    let set_clause = set_clauses.join(", ");
    let query = format!(
        "UPDATE bugReport SET {} WHERE id = ? RETURNING *",
        set_clause
    );

    // Add bug_id to blob_params LAST
let bug_id_bytes = _bug_id.into_inner().as_bytes().to_vec();

// Build and bind query
let mut query_builder = sqlx::query_as::<_, BugReport>(&query);

// Bind string parameters first
for param in &string_params {
    query_builder = query_builder.bind(*param);
}

// Bind blob parameters next (except bug_id)
for param in &blob_params {
    query_builder = query_builder.bind::<Vec<u8>>(param.clone());
}

// Finally bind bug_id
query_builder = query_builder.bind(bug_id_bytes);

    // Execute query
    match query_builder.fetch_one(_pool.get_ref()).await {
        Ok(updated_bug) => HttpResponse::Ok().json(updated_bug),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to update bug")
        }
    }
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn delete_bug(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
}