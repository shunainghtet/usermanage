#[macro_use] extern crate rocket;

use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::http::Status;
use std::sync::Mutex;
use std::collections::HashMap;

// Define a struct for User
#[derive(Debug, Deserialize, Serialize, Clone)]
struct User {
    id: u32,
    username: String,
    email: String,
    phone_no: String,
    password: String,
}

// Define a struct for input (update data)
#[derive(Debug, Deserialize)]
struct UpdateUserInput {
    username: Option<String>,
    email: Option<String>,
    phone_no: Option<String>,
    password: Option<String>,
}

// Shared state for storing users
#[derive(Default)]
struct AppState {
    users: Mutex<HashMap<u32, User>>,
}

// POST: Create a new user
#[post("/users", format = "json", data = "<user_input>")]
fn create_user(user_input: Json<User>, state: &rocket::State<AppState>) -> (Status, Json<String>) {
    let mut users = state.users.lock().unwrap();

    if users.contains_key(&user_input.id) {
        return (Status::Conflict, Json(format!("User with ID {} already exists.", user_input.id)));
    }

    users.insert(user_input.id, user_input.into_inner());
    (Status::Created, Json(format!("User created successfully!")))
}

// PUT: Update user details
#[put("/users/<id>", format = "json", data = "<update_data>")]
fn update_user(
    id: u32,
    update_data: Json<UpdateUserInput>,
    state: &rocket::State<AppState>,
) -> (Status, Json<String>) {
    let mut users = state.users.lock().unwrap();

    // Find the user
    if let Some(user) = users.get_mut(&id) {
        // Update the fields if present
        if let Some(username) = &update_data.username {
            user.username = username.clone();
        }
        if let Some(email) = &update_data.email {
            user.email = email.clone();
        }
        if let Some(phone_no) = &update_data.phone_no {
            user.phone_no = phone_no.clone();
        }
        if let Some(password) = &update_data.password {
            user.password = password.clone(); // Hash in production
        }

        return (Status::Ok, Json(format!("User with ID {} updated successfully.", id)));
    }

    // If user not found
    (Status::NotFound, Json(format!("User with ID {} not found.", id)))
}

// GET: Fetch all users (for testing)
#[get("/users")]
fn list_users(state: &rocket::State<AppState>) -> Json<Vec<User>> {
    let users = state.users.lock().unwrap();
    Json(users.values().cloned().collect())
}

// Rocket entry point
#[launch]
fn rocket() -> _ {
    let state = AppState::default();
    rocket::build()
        .manage(state)
        .mount("/api", routes![create_user, update_user, list_users])
}
