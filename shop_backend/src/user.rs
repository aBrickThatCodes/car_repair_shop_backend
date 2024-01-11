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
    id: u32,
    name: String,
    user_type: UserType,
}

impl User {
    pub(crate) fn logged_in(id: u32, name: &str, user_type: UserType) -> Self {
        assert!(!matches!(&user_type, UserType::NotLoggedIn));
        User {
            id,
            name: name.to_string(),
            user_type,
        }
    }

    pub(crate) fn not_logged_in() -> Self {
        User {
            id: 0,
            name: String::new(),
            user_type: UserType::NotLoggedIn,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn user_type(&self) -> UserType {
        self.user_type.clone()
    }
}
