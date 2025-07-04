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

// Models and authentication functionality needed for user, stock, and transaction handling.
use crate::models::{User, BugReport, LoginRequest, LoginResponse, CreateBug, ProjectRecord, BugAssignment, SimpleUser, BugFilter, UpdateBugReport, CreateProject};
use crate::auth;

// Function to configure the service, setting up the routes available in this web application.
pub fn config(cfg: &mut web::ServiceConfig) {
    // Public routes (no authentication middleware applied)
    cfg.service(web::resource("/login").route(web::post().to(login_function))) // User login (public)
       .service(web::resource("/projects").route(web::get().to(get_projects))) // GET /projects (public)
       .service(
           web::scope("/bugs") // Public GET routes for bugs
               .route("", web::get().to(get_bugs)) // GET /bugs (public)
               .route("/assign", web::get().to(render_bug_form)) // GET /bugs/assign (public)
               .route("/{id}", web::get().to(get_bug_by_id)) // GET /bugs/{id} (public)
       );

    // Authenticated routes (authentication middleware applied)
    cfg.service(
        web::scope("") // This scope wraps all routes that require authentication
            .wrap(auth::AuthMiddleware) // Apply the AuthMiddleware here
            .service(web::resource("/projects").route(web::post().to(add_project))) // POST /projects (authenticated)
            .service(
                web::scope("/bugs") // Authenticated routes for bugs
                    .route("/assign", web::post().to(assign_bug)) // POST /bugs/assign (authenticated)
                    .route("/new", web::post().to(create_bug)) // POST /bugs/new (authenticated)
                    .route("/{id}", web::patch().to(update_bug_details)) // PATCH /bugs/{id} (authenticated)
                    .route("/{id}", web::delete().to(delete_bug)) // DELETE /bugs/{id} (authenticated)
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
async fn get_projects(_pool: web::Data<SqlitePool>) -> impl Responder {
    let project = match sqlx::query_as::<_, ProjectRecord>(
        "SELECT id, project_name, project_description, created_at, user_id  FROM projectRecords"
    )
    .fetch_all(_pool.get_ref())
    .await
    {
        Ok(projects) => projects,
        Err(e) => {
            eprintln!("Project query error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(project)
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn add_project(_pool: web::Data<SqlitePool>, _body: web::Json<CreateProject>) -> impl Responder {
    // Query to get the user by username
    let user = match sqlx::query_as::<_, User>(
        "SELECT id, username, hashed_password FROM users WHERE username = ?"
    )
    .bind(&_body.username)
    .fetch_optional(_pool.get_ref())
    .await {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::NotFound().body("User not found"),
        Err(e) => {
            eprintln!("User query error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Generate a new UUID for the project
    let project_id = Uuid::new_v4();
    let user_id = user.id.clone(); // Get the user's id

    // Insert the new project into the database
    let project = match sqlx::query(
        "INSERT INTO projectRecords (id, user_id, project_name, project_description) VALUES (?, ?, ?, ?)"
    )
    .bind(&project_id)
    .bind(&user_id) // Binding user_id from the User struct
    .bind(&_body.project_title)
    .bind(&_body.project_description)
    .execute(_pool.get_ref())
    .await {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Create project error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(project)
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
    let bug_id = _bug_id.into_inner();

    // Convert Uuid to Vec<u8> for matching BLOB field in SQLite
    let bug_id_bytes = bug_id.as_bytes().to_vec();

    match sqlx::query_as::<_, BugReport>(
        "SELECT id, project_id, title, description, reported_by, fixed_by, severity, is_fixed, created_at \
         FROM bugReport WHERE id = ?"
    )
    .bind(bug_id_bytes)
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

// Simply responds to the request with a confirmation message.
async fn create_bug(_pool: web::Data<SqlitePool>, _body: web::Json<CreateBug>,_req: HttpRequest) -> impl Responder {
    // Extract user ID from the request extensions
    let authenticated_user_id = match auth::get_authenticated_user_id(&_req) {
        Some(user_id) => user_id,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({
            "status": "error",
            "message": "Authentication required"
        }))
    };

    // Get the authenticated user from database
    let user = match sqlx::query_as::<_, User>(
        "SELECT id, username, hashed_password FROM users WHERE id = ?"
    )
    .bind(&authenticated_user_id)
    .fetch_optional(_pool.get_ref())
    .await {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::Unauthorized().json(serde_json::json!({
            "status": "error",
            "message": "User not found"
        })),
        Err(e) => {
            eprintln!("User query error: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Database error"
            }));
        }
    };
  
    // Get project by name
    let project = match sqlx::query_as::<_, ProjectRecord>(
        "SELECT id, project_name, project_description, created_at, user_id FROM projectRecord WHERE project_name = ?"
    )
    .bind(&_body.project_name)
    .fetch_optional(_pool.get_ref())
    .await {
        Ok(Some(project)) => project,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({
            "status": "error",
            "message": "Project not found"
        })),
        Err(e) => {
            eprintln!("Project query error: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Database error"
            }));
        }
    };

    let bug_id = uuid::Uuid::new_v4();
    
    // Insert bug report using authenticated user's ID
    if let Err(e) = sqlx::query("INSERT INTO bugReport (id, project_id, title, description, reported_by, severity, is_fixed) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&bug_id.as_bytes().as_slice())// Convert UUID to bytes for SQLite
        .bind(&project.id.as_bytes().as_slice())
        .bind(&_body.title)
        .bind(&_body.description)
        .bind(&user.id.as_bytes().as_slice())
        .bind(&_body.severity)
        .bind(false)
        .execute(_pool.get_ref())
        .await
    {
        eprintln!("BugReport insert error: {:?}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": "Failed to create bug report"
        }));
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


async fn delete_bug(_pool: web::Data<SqlitePool>, _bug_id: web::Path<Uuid>) -> impl Responder {
    let bug_id = _bug_id.into_inner();

    // Convert Uuid to Vec<u8> for matching BLOB field in SQLite
    let bug_id_bytes = bug_id.as_bytes().to_vec();

    match sqlx::query("DELETE FROM bugReport WHERE id = ?")
        .bind(bug_id_bytes)
        .execute(_pool.get_ref())
        .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                HttpResponse::NotFound().body("Bug not found")
            } else {
                HttpResponse::Ok().body("Bug deleted successfully")
            }
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to delete bug")
        }
    }
}