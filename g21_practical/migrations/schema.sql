-- Create users table
CREATE TABLE users (
    id BLOB PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    hashed_password TEXT NOT NULL
);

-- Create projectRecord table
CREATE TABLE projectRecord (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL,
    project_name TEXT,
    project_description TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(user_id) REFERENCES users(id)
);

-- Create bugsReports table
CREATE TABLE bugReport (
    id BLOB PRIMARY KEY,
    title TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    project_id BLOB NOT NULL,
    reported_by BLOB NOT NULL,
    fixed_by BLOB,
    severity TEXT NOT NULL,
    is_fixed BOOLEAN DEFAULT FALSE,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(project_id) REFERENCES projectRecord(id),
    FOREIGN KEY(reported_by) REFERENCES users(id),
    FOREIGN KEY(fixed_by) REFERENCES users(id)
);
