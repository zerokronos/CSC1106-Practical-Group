// Import necessary modules and types from external crates and internal modules.
// `actix_web` provides tools for building web servers, including handling HTTP requests and responses.
// `sqlx` is used for database connection pooling. `SqlitePool` is a specific pool type for SQLite.
// `uuid` is used for generating unique identifiers.
// `crate::models` and `crate::auth` denote relative imports from the current project's `models` and `auth` modules, respectively.
use actix_web::{web, HttpResponse, Responder};
use sqlx::SqlitePool;
use uuid::Uuid;

// Models and authentication functionality needed for user, stock, and transaction handling.
use crate::models::{BugReport, CreateBug, CreateProject, Project, User};
use crate::auth;

// Define a function to configure the service, setting up the routes available in this web application.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login_function))) // POST /login 
        .service(
            web::scope("/projects")
                .route("", web::get().to(get_projects))
                .route("", web::post().to(add_project))
        )  
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
async fn login_function(_pool: web::Data<SqlitePool>, body: web::Json<User>) -> impl Responder {
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
async fn get_projects(_pool: web::Data<SqlitePool>) -> impl Responder {
    let project = match sqlx::query_as::<_, Project>(
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

    let project = match sqlx::query_as::<_, Project>(
        "SELECT id, project_name, project_description, created_at, user_id  FROM projectRecords WHERE project_name = ?"
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
    
    if let Err(e) = sqlx::query("INSERT INTO bugReports (id, project_id, title, description, reported_by, severity, is_fixed) VALUES (?, ?, ?, ?, ?, ?, ?)")
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
        fixed_by: uuid::Uuid::nil(), // Initially set to nil, as the bug is not fixed yet
        created_at: chrono::Utc::now().to_rfc3339(), // Current timestamp in RFC 3339 format
        is_fixed: false,
    };

    HttpResponse::Ok().json(response)
}


// Asynchronous function for handling stock purchase requests.
// Simply responds to the request with a confirmation message.
async fn assign_bug(_pool: web::Data<SqlitePool>, _body: web::Json<BugReport>) -> impl Responder {
    // Respond with a 200 OK status, indicating the buy request was processed.
    HttpResponse::Ok().body("Buy request processed")
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