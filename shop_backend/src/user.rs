#[derive(Clone)]
pub enum User {
    Client {
        id: i32,
        email: String,
        name: String,
    },
    Technician {
        id: i32,
        name: String,
    },
    Mechanic {
        id: i32,
        name: String,
    },
    NotLoggedIn,
}
