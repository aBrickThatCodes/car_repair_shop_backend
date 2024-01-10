#[derive(Clone, Debug)]
pub enum UserType {
    Client,
    Technician,
    Mechanic,
    NotLoggedIn,
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

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn user_type(&self) -> UserType {
        self.user_type.clone()
    }
}
