# G21 GroupProject

## Overview
Our program uses Actix Web as the framework and SQLite for data storage. Key features include authentication using middleware JWT Token, API's for accessing various tables in 
the database implemented in Rust as well as a frontend using Tera template with simple HTML/CSS and Javascript.

## Project Structure
├── g21_practical
    ├── curl
        ├── commands                # Commonly used CLI
    ├── migrations                  
        ├── schema.sql              # Database Schema
    ├── src
        ├── auth.rs                 # Authentication logic
        ├── db.rs                   # SQLite Database logic
        ├── handlers.rs             # API route logic
        ├── main.rs                 # Entry point
        ├── models.rs               # Data models
    ├── static
        ├── bugform.html            # Frontend
    ├── Cargo.toml                  # Rust dependencies
    ├── README.txt                  # Project documentation

## Installation & Setup
### Prerequisites
    - Install Rust: [rust-lang.org](https://www.rust-lang.org/tools/install)

### Setup Instructions
   cd g21_practical
   cargo run

## Usage of Application
1. **Web Interface**:   Open 'http://localhost:8080/bugs/assign' or bound ip address and port.
2. **CLI API Route**:   Open curl/commands to view the respective commands to test the API Routes.     

## Features
-Authentication Middleware with JWT Token

-Hashing of password with Salt via Bcrypt

-Front end with Tera Template

-SQLite with in-memory database.
    For our SQLite Database, we used an in-memory database to store our tables and data.
    schema.sql is called in db.rs to generate the tables (users, projectRecord, bugReport, etc.)
    The tables are then populated with some data for testing.

    Key constrains of the ralations are
    Each projectReport has a user (identified with user_id) that created it
    Each bugReport has a project (identified with project_id), user (identified with reported_by) that reported it, and a user (identified with fixed_by) that fixed it.
    
-Error Handling with error.rs

-CRUD
    create_bug takes in title, description, project_name, and severity. It automatically checks the projectname with the database and binds respective fields.
    It returns the newly created BugReport as a json.

    get_bugs takes in optional fields of is_fixed, severity and project_name as queries. It will return all selected BugReports based on the filters as a JSON.

    get_bug_by_id takes in a bug_id in its path and returns all fields of the BugReport from the database in a JSON.

    update_bug_details takes in optional fields of is_fixed, severity, description and fixed_by and updates the respective fields of the bug_id which is passed
    in through the path.

    delete_bug deletes a BugReport with the assigned bug_id passed in through the path
    
## API Routes
**POST** `/login` - login as a user
**GET** `/projets` - Get all projects as JSON
**POST** `/projects` - Add a new project (require Authentication)

## FrontEnd API Routes
**GET** `/bugs/assign` - Renders the field in Tera HTML template
**POST** `/bugs/assign` - Update the fixed by field(require Authentication)

## CRUD API for BugReport
**POST** `/bugs/new` - Create a new BugReport (require Authentication)
**GET** `/bugs` - List all BugReport's as a JSON
**GET** `/bugs/:id` - Retrive a specific BugReport by bug_id as JSON
**PATCH** `/bugs/:id` - Update BugReport details via JSON with optional fields, returns updated record (require Authentication)
**DELETE** `/bugs/:id` - Delete a BugReport by bug_id (require Authentication)
