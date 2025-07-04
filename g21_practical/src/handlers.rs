// Import necessary modules and types from external crates and internal modules.
// `actix_web` provides tools for building web servers, including handling HTTP requests and responses.
// `sqlx` is used for database connection pooling. `SqlitePool` is a specific pool type for SQLite.
// `uuid` is used for generating unique identifiers.
// `crate::models` and `crate::auth` denote relative imports from the current project's `models` and `auth` modules, respectively.
use actix_web::{web, HttpResponse, Responder, HttpRequest, Error, Result};
use sqlx::SqlitePool;
use uuid::Uuid;
use hex;
use tera::{Tera, Context};

use crate::models::{User, BugReport, LoginRequest, LoginResponse, CreateBug, ProjectRecord, BugAssignment, SimpleUser, BugFilter, UpdateBugReport, CreateProject};
use crate::auth;
use crate::error::AppError;

// Function to configure the service, setting up the routes available in this web application.
pub fn config(cfg: &mut web::ServiceConfig) {
    // The login route is a standalone public endpoint
    cfg.service(web::resource("/login").route(web::post().to(login_function)));

    // Configure all /projects routes within a single scope
    cfg.service(
        web::scope("/projects")
            // Public GET /projects
            .route("", web::get().to(get_projects))
            // Authenticated POST /projects (add_project)
            // This nested service ensures only the POST method for /projects is authenticated
            .service(
                web::resource("") // Relative path to /projects, so it matches /projects
                    .wrap(auth::AuthMiddleware) // Apply middleware only to this resource
                    .route(web::post().to(add_project))
            )
    );

    // Configure all /bugs routes within a single top-level scope
    cfg.service(
        web::scope("/bugs")
            // Public GET routes for /bugs
            .route("", web::get().to(get_bugs))
            .route("/assign", web::get().to(render_bug_form))
            .route("/{id}", web::get().to(get_bug_by_id))

            // Nested scope for authenticated /bugs operations (POST, PATCH, DELETE)
            // This inner scope inherits the "/bugs" prefix from its parent.
            .service(
                web::scope("") // This effectively means "/bugs/*"
                    .wrap(auth::AuthMiddleware) // Apply AuthMiddleware to all routes within this nested scope
                    // Authenticated POST /bugs/assign
                    .route("/assign", web::post().to(assign_bug))
                    // Authenticated POST /bugs/new
                    .route("/new", web::post().to(create_bug))
                    // Authenticated PATCH /bugs/{id}
                    .route("/{id}", web::patch().to(update_bug_details))
                    // Authenticated DELETE /bugs/{id}
                    .route("/{id}", web::delete().to(delete_bug))
            )
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
                    let token = auth::create_token(user.id);

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
async fn get_projects(_pool: web::Data<SqlitePool>) -> Result<impl Responder, AppError> {
    let project = sqlx::query_as::<_, ProjectRecord>(
        "SELECT id, project_name, project_description, created_at, user_id  FROM projectRecord"
    )
    .fetch_all(_pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Project query error: {:?}", e);
        AppError::Database(e.into())
    })?;

    Ok(HttpResponse::Ok().json(project))
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn add_project(_pool: web::Data<SqlitePool>, _body: web::Json<CreateProject>) -> Result<impl Responder, AppError> {
    // Query to get the user by username
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, hashed_password FROM users WHERE username = ?"
    )
    .bind(&_body.username)
    .fetch_optional(_pool.get_ref())
    .await
    .map_err(|e| { 
        eprintln!("User query error: {:?}", e); 
        AppError::Database(e.into())
    })? 
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Generate a new UUID for the project
    let project_id = Uuid::new_v4();
    let user_id = user.id.clone(); // Get the user's id

    // Insert the new project into the database
    let project = sqlx::query(
        "INSERT INTO projectRecord (id, user_id, project_name, project_description) VALUES (?, ?, ?, ?)"
    )
    .bind(&project_id)
    .bind(&user_id) // Binding user_id from the User struct
    .bind(&_body.project_title)
    .bind(&_body.project_description)
    .execute(_pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Create project error: {:?}", e);
        AppError::Database(e.into())
    })?;

    Ok(HttpResponse::Ok().body("Project added successfully"))
}

// Asynchronous function for fetching bug reports based on optional filters.
async fn get_bugs(_pool: web::Data<SqlitePool>, _filter: web::Query<BugFilter>) -> Result<impl Responder, AppError> {
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
        let project = sqlx::query_as::<_, ProjectRecord>(
            "SELECT id, project_name, project_description, created_at, user_id FROM projectRecord WHERE project_name = ?"
        )
        .bind(project_name)
        .fetch_optional(_pool.get_ref())
        .await
        .map_err(|e| {
            eprintln!("Error fetching project: {:?}", e);
            AppError::Database(e.into())
        })? 
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        // Now add the project_id condition to the query
        let project_id_bytes = project.id.as_bytes().to_vec();
        query.push_str(&format!(" AND project_id = x'{}'", hex::encode(project_id_bytes)));
    }

    // Execute the final query
    let bugs = sqlx::query_as::<_, BugReport>(&query).fetch_all(_pool.get_ref()).await
        .map_err(|e| {
            eprintln!("Database error: {:?}", e);
            AppError::Database(e.into())
        })?;

    Ok(HttpResponse::Ok().json(bugs))
}

// Asynchronous function for getting a bug by its ID in it path.
async fn get_bug_by_id(_pool: web::Data<SqlitePool>, _bug_id: web::Path<String>) -> Result<impl Responder, AppError> {
    // Manually parse the UUID string.
    // If parsing fails, return an AppError::BadRequest.
    let bug_id = Uuid::parse_str(&_bug_id.into_inner())
        .map_err(|e| {
            eprintln!("UUID parsing failed: {:?}", e);
            AppError::BadRequest(format!("Invalid Bug ID format: {}", e))
        })?;

    // Convert Uuid to Vec<u8> for matching BLOB field in SQLite
    let bug_id_bytes = bug_id.as_bytes().to_vec();

    let bug = sqlx::query_as::<_, BugReport>(
        "SELECT id, project_id, title, description, reported_by, fixed_by, severity, is_fixed, created_at \
         FROM bugReport WHERE id = ?"
    )
    .bind(bug_id_bytes)
    .fetch_optional(_pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error fetching bug by ID: {:?}", e);
        AppError::Database(e.into())
    })?
    .ok_or_else(|| AppError::NotFound("Bug not found".to_string()))?;

    Ok(HttpResponse::Ok().json(bug))
}


// Asynchronous function for creating a new bug report.
// Simply responds to the request with a confirmation message.
async fn create_bug(_pool: web::Data<SqlitePool>, _body: web::Json<CreateBug>,_req: HttpRequest) -> Result<impl Responder, AppError> {
    // Extract user ID from the request extensions
    let authenticated_user_id = match auth::get_authenticated_user_id(&_req) {
        Some(user_id) => user_id,
        None => return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "status": "error",
            "message": "Authentication required"
        })))
    };

    // Get the authenticated user from database
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, hashed_password FROM users WHERE id = ?"
    )
    .bind(&authenticated_user_id)
    .fetch_optional(_pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("User query error: {:?}", e);
        AppError::Database(e.into())
    })? 
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
  
    // Get project by name
    let project = sqlx::query_as::<_, ProjectRecord>(
        "SELECT id, project_name, project_description, created_at, user_id FROM projectRecord WHERE project_name = ?"
    )
    .bind(&_body.project_name)
    .fetch_optional(_pool.get_ref())
    .await
    .map_err(|e| { 
        eprintln!("Project query error: {:?}", e); 
        AppError::Database(e.into()) 
    })? 
    .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    let bug_id = uuid::Uuid::new_v4();
    
    // Insert bug report using authenticated user's ID
    sqlx::query("INSERT INTO bugReport (id, project_id, title, description, reported_by, severity, is_fixed) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&bug_id.as_bytes().as_slice())// Convert UUID to bytes for SQLite
        .bind(&project.id.as_bytes().as_slice())
        .bind(&_body.title)
        .bind(&_body.description)
        .bind(&user.id.as_bytes().as_slice())
        .bind(&_body.severity)
        .bind(false)
        .execute(_pool.get_ref())
        .await
        .map_err(|e| { 
            eprintln!("BugReport insert error: {:?}", e);
            AppError::Database(e.into())
        })?;

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

    Ok(HttpResponse::Ok().json(response))
}


// Asynchronous function to render the bug assignment form.
async fn render_bug_form(pool: web::Data<SqlitePool>) -> Result<impl Responder, AppError> {
    println!("render_bug_form called");
    
    // Fetch open bugs
    let open_bugs = sqlx::query_as::<_, BugReport>(
        "SELECT * FROM bugReport WHERE is_fixed = false"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| { 
        eprintln!("Error fetching bugs from database: {:?}", e); 
        AppError::Database(e.into()) 
    })?;

    // Fetch all users
    let users = sqlx::query_as::<_, SimpleUser>(
        "SELECT id, username FROM users"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| { 
        eprintln!("Error fetching users from database: {:?}", e); 
        AppError::Database(e.into()) 
    })?;

    println!("Found {} bugs and {} users", open_bugs.len(), users.len());

    //For testing purposes
    for bug in &open_bugs {
                println!("Bug ID: {}, Title: {}", bug.id, bug.title);
            }
    for user in &users {
                println!("UserID: {}, Username: {}", user.id, user.username);
            }
            
    // Create Tera instance
    let tera = match Tera::new("static/*.html") {
        Ok(t) => {
            println!("Tera instance created successfully");
            t
        },
        Err(e) => {
            eprintln!("Tera parsing error: {}", e);
            return Ok(HttpResponse::InternalServerError().body("Template parsing error")); // Not an AppError variant
        }
    };

    let mut context = Context::new(); 
    context.insert("bugs", &open_bugs); 
    context.insert("users", &users); 

    // Render the template
    match tera.render("bugform.html", &context) {
        Ok(rendered) => {
            println!("Template rendered successfully"); 
            Ok(HttpResponse::Ok().content_type("text/html").body(rendered)) 
        },
        Err(e) => {
            eprintln!("Template rendering error: {}", e); 
            Ok(HttpResponse::InternalServerError().body("Failed to render template")) // Not an AppError variant
        }
    }
}

// Error handling not yet placed
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


// Asynchronous function to update a bug details.
pub async fn update_bug_details(_pool: web::Data<SqlitePool>,_bug_id: web::Path<String>,_body: web::Json<UpdateBugReport>) -> Result<impl Responder, AppError> {

    // Manually parse the UUID string.
    // If parsing fails, return an AppError::BadRequest.
    let bug_id = Uuid::parse_str(&_bug_id.into_inner())
        .map_err(|e| {
            eprintln!("UUID parsing failed for update_bug_details: {:?}", e);
            AppError::BadRequest(format!("Invalid Bug ID format: {}", e))
        })?;
    
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
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, hashed_password FROM users WHERE username = ?",
        )
        .bind(fixed_by_username)
        .fetch_optional(_pool.get_ref())
        .await
        .map_err(|e| { 
            eprintln!("Database error: {:?}", e); 
            AppError::Database(e.into()) 
        })? //
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?; 

        set_clauses.push("fixed_by = ?"); 
        blob_params.push(user.id.as_bytes().to_vec()); 
    }

    if set_clauses.is_empty() { 
        return Err(AppError::BadRequest("No fields provided for update".to_string())); 
    }

    // Build final query
    let set_clause = set_clauses.join(", "); 
    let query = format!(
        "UPDATE bugReport SET {} WHERE id = ? RETURNING *",
        set_clause
    );

    // Add bug_id to blob_params LAST
    let bug_id_bytes = bug_id.as_bytes().to_vec(); 

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
    let updated_bug = query_builder.fetch_one(_pool.get_ref()).await //
        .map_err(|e| { 
            eprintln!("Database error: {:?}", e); 
            AppError::Database(e.into()) 
        })?; 

    Ok(HttpResponse::Ok().json(updated_bug))
}


async fn delete_bug(_pool: web::Data<SqlitePool>, _bug_id: web::Path<String>) -> Result<impl Responder, AppError> {
    // Manually parse the UUID string.
    // If parsing fails, return an AppError::BadRequest.
    let bug_id = Uuid::parse_str(&_bug_id.into_inner())
        .map_err(|e| {
            eprintln!("UUID parsing failed for delete_bug: {:?}", e);
            AppError::BadRequest(format!("Invalid Bug ID format: {}", e))
        })?;

    // Convert Uuid to Vec<u8> for matching BLOB field in SQLite
    let bug_id_bytes = bug_id.as_bytes().to_vec();

    let result = sqlx::query("DELETE FROM bugReport WHERE id = ?")
        .bind(bug_id_bytes)
        .execute(_pool.get_ref())
        .await
        .map_err(|e| { 
            eprintln!("Database error: {:?}", e); 
            AppError::Database(e.into()) 
        })?; 

    if result.rows_affected() == 0 { 
        Err(AppError::NotFound("Bug not found".to_string())) 
    } else {
        Ok(HttpResponse::Ok().body("Bug deleted successfully")) 
    }
}