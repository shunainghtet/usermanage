// Appointment Add,View and Cancel
#[macro_use]
extern crate rocket;

use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::http::Status;
use std::sync::Mutex;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Appointment {
    id: u32,
    name: String,
    email: String,
    phone: String,
    date: String,
    time: String,
    reason: String,
}

#[derive(Default)]
struct AppState {
    appointments: Mutex<HashMap<u32, Appointment>>,
    next_id: Mutex<u32>, // To generate unique IDs for appointments
}

// Schedule a new appointment
#[post("/appointments", format = "json", data = "<appointment_input>")]
fn schedule_appointment(
    appointment_input: Json<AppointmentRequest>,
    state: &rocket::State<AppState>,
) -> (Status, Json<String>) {
    let mut appointments = state.appointments.lock().unwrap();
    let mut next_id = state.next_id.lock().unwrap();

    let appointment = Appointment {
        id: *next_id,
        name: appointment_input.name.clone(),
        email: appointment_input.email.clone(),
        phone: appointment_input.phone.clone(),
        date: appointment_input.date.clone(),
        time: appointment_input.time.clone(),
        reason: appointment_input.reason.clone(),
    };

    appointments.insert(*next_id, appointment);
    *next_id += 1;

    (
        Status::Created,
        Json(format!("Appointment scheduled successfully with ID {}", *next_id - 1)),
    )
}

// View an appointment by ID
#[get("/appointments/<id>")]
fn view_appointment(id: u32, state: &rocket::State<AppState>) -> Option<Json<Appointment>> {
    let appointments = state.appointments.lock().unwrap();
    appointments.get(&id).map(|appointment| Json(appointment.clone()))
}

// List all appointments
#[get("/appointments")]
fn list_appointments(state: &rocket::State<AppState>) -> Json<Vec<Appointment>> {
    let appointments = state.appointments.lock().unwrap();
    Json(appointments.values().cloned().collect())
}

// Cancel an appointment
#[delete("/appointments/<id>")]
fn cancel_appointment(id: u32, state: &rocket::State<AppState>) -> (Status, Json<String>) {
    let mut appointments = state.appointments.lock().unwrap();
    if appointments.remove(&id).is_some() {
        (
            Status::Ok,
            Json(format!("Appointment with ID {} canceled successfully.", id)),
        )
    } else {
        (
            Status::NotFound,
            Json(format!("No appointment found with ID {}.", id)),
        )
    }
}

// Request data structure for scheduling an appointment
#[derive(Debug, Deserialize)]
struct AppointmentRequest {
    name: String,
    email: String,
    phone: String,
    date: String,
    time: String,
    reason: String,
}

#[launch]
fn rocket() -> _ {
    let state = AppState::default();
    rocket::build()
        .manage(state)
        .mount(
            "/api",
            routes![
                schedule_appointment,
                view_appointment,
                list_appointments,
                cancel_appointment
            ],
        )
}
