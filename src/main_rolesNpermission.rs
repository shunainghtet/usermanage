#[macro_use] extern crate rocket;

use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::http::Status;
use std::sync::Mutex;
use std::collections::{HashMap, HashSet};

// Define a struct for Permissions
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
enum Permission {
    ViewPatient,
    AddPatient,
    EditPatient,
    DeletePatient,
    ViewDoctor,
    AddDoctor,
}

// Define a struct for Role
#[derive(Debug, Clone, Serialize, Deserialize)]
enum Role {
    Admin,
    Doctor,
    Nurse,
}

// Define a struct for User
#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: u32,
    username: String,
    role: Role,
    permissions: HashSet<Permission>,
}

// In-memory storage for Users
#[derive(Default)]
struct AppState {
    users: Mutex<HashMap<u32, User>>,
}

impl User {
    // Check if the user has a certain permission
    fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
}

// Create a user
#[post("/users", format = "json", data = "<user_input>")]
fn create_user(user_input: Json<User>, state: &rocket::State<AppState>) -> (Status, Json<String>) {
    let mut users = state.users.lock().unwrap();
    if users.contains_key(&user_input.id) {
        return (Status::Conflict, Json(format!("User with ID {} already exists.", user_input.id)));
    }

    // Assign default permissions based on role
    let mut permissions = HashSet::new();
    match &user_input.role {
        Role::Admin => {
            permissions.insert(Permission::ViewPatient);
            permissions.insert(Permission::AddPatient);
            permissions.insert(Permission::EditPatient);
            permissions.insert(Permission::DeletePatient);
            permissions.insert(Permission::ViewDoctor);
            permissions.insert(Permission::AddDoctor);
        }
        Role::Doctor => {
            permissions.insert(Permission::ViewPatient);
            permissions.insert(Permission::AddPatient);
            permissions.insert(Permission::EditPatient);
            permissions.insert(Permission::ViewDoctor);
        }
        Role::Nurse => {
            permissions.insert(Permission::ViewPatient);
        }
    }

    // Move the user_input and modify the user
    let mut new_user = user_input.into_inner();
    new_user.permissions = permissions;
    
    // Insert the user and clone for response
    users.insert(new_user.id, new_user.clone());

    // Format response with the cloned user for borrowing
    (Status::Created, Json(format!("User '{}' created successfully!", new_user.username)))
}

// Update user role
#[put("/users/<id>/role", format = "json", data = "<new_role>")]
fn update_user_role(id: u32, new_role: Json<Role>, state: &rocket::State<AppState>) -> (Status, Json<String>) {
    let mut users = state.users.lock().unwrap();
    if let Some(user) = users.get_mut(&id) {
        user.role = new_role.into_inner();
        user.permissions.clear();  // Clear previous permissions
        match &user.role {
            Role::Admin => {
                user.permissions.insert(Permission::ViewPatient);
                user.permissions.insert(Permission::AddPatient);
                user.permissions.insert(Permission::EditPatient);
                user.permissions.insert(Permission::DeletePatient);
                user.permissions.insert(Permission::ViewDoctor);
                user.permissions.insert(Permission::AddDoctor);
            }
            Role::Doctor => {
                user.permissions.insert(Permission::ViewPatient);
                user.permissions.insert(Permission::AddPatient);
                user.permissions.insert(Permission::EditPatient);
                user.permissions.insert(Permission::ViewDoctor);
            }
            Role::Nurse => {
                user.permissions.insert(Permission::ViewPatient);
            }
        }
        return (Status::Ok, Json(format!("User {} role updated to {:?}", id, user.role)));
    }

    (Status::NotFound, Json(format!("User with ID {} not found.", id)))
}

// Assign permission to a user
#[put("/users/<id>/permissions", format = "json", data = "<permissions>")]
fn assign_permissions(id: u32, permissions: Json<HashSet<Permission>>, state: &rocket::State<AppState>) -> (Status, Json<String>) {
    let mut users = state.users.lock().unwrap();
    if let Some(user) = users.get_mut(&id) {
        user.permissions.extend(permissions.into_inner());
        return (Status::Ok, Json(format!("Permissions assigned to user {}.", id)));
    }

    (Status::NotFound, Json(format!("User with ID {} not found.", id)))
}

// View user details
#[get("/users/<id>")]
fn get_user(id: u32, state: &rocket::State<AppState>) -> Option<Json<User>> {
    let users = state.users.lock().unwrap();
    users.get(&id).map(|user| Json(user.clone()))
}

// List all users
#[get("/users")]
fn list_users(state: &rocket::State<AppState>) -> Json<Vec<User>> {
    let users = state.users.lock().unwrap();
    Json(users.values().cloned().collect())
}

#[launch]
fn rocket() -> _ {
    let state = AppState::default();
    rocket::build()
        .manage(state)
        .mount("/api", routes![create_user, update_user_role, assign_permissions, get_user, list_users])
}
