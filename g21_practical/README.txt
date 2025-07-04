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
   cargo build
   cargo run

## Usage of Application
1. **Web Interface**:   Open 'http://localhost:8080/bugs/assign' or bound ip address and port.
2. **CLI API Route**:       Open curl/commands to use the respective commands to test the API Routes.     

## Features
-Authentication Middleware with JWT Token

-Hashing of password with Salt

-Front end with Tera Template

-SQLite with in-memory database.
    For our SQLite Database we used an in memory database to store our tables and data.
    schema.sql is called in db.rs to generate the tables (users, projectRecord, BugReport)
    The tables are then populated with some data for tessting.

    Key contrains of the realations are
    Each projectReport has a user (identified with user_id) that created it
    Each bugReport has a project (identified with project_id), user (identified with reported_by) that reported it, and a user (identified with fixed_by) that fixed it.
    
-Error Handling


## API Routes
**POST** `/login` - login as a user
**GET** `/projets` - Get all projects as JSON
**POST** `/projects` - Add a new project (Admin Access required)
**GET** `/bugs/assign` - Renders the Tera HTML template
**POST** `/bugs/assign` - Update the fixed by field
**POST** `/bugs/new` - Creat a new BugReport 
**GET** `/bugs` - List all BugReport's as a JSON
**GET** `/bugs/:id` - Retrive a specific BugReport by bug_id as JSON
**PATCH** `/bugs/:id` - Update BugReport details via JSON with optional fields (such as status, assigned developer, severity, description), returns updated record
**DELETE** `/bugs/:id` - Delete a BugReport by bug_id
