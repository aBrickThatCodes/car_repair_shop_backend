#[derive(Clone, Debug)]
pub enum UserType {
    Client,
    Technician,
    Mechanic,
    NotLoggedIn,
}

impl std::fmt::Display for UserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            UserType::Client => "Client",
            UserType::Technician => "Technician",
            UserType::Mechanic => "Mechanic",
            UserType::NotLoggedIn => "Not logged in",
        })
    }
}

#[derive(Clone, Debug)]
pub struct User {
    id: i32,
    name: String,
    user_type: UserType,
}

impl User {
    pub fn logged_in(id: i32, name: &str, user_type: UserType) -> Self {
        assert!(!matches!(&user_type, UserType::NotLoggedIn));
        User {
            id,
            name: name.to_string(),
            user_type,
        }
    }

    pub fn not_logged_in() -> Self {
        User {
            id: -1,
            name: String::new(),
            user_type: UserType::NotLoggedIn,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn user_type(&self) -> UserType {
        self.user_type.clone()
    }
}
