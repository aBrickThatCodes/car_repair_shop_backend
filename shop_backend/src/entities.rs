use crate::{db_entities::employee::Role, Car, Service};

use serde::{Deserialize, Serialize};

use std::fmt::Display;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Client {
    id: u32,
    name: String,
    email: String,
    car: Option<Car>,
}

impl Client {
    pub fn new(id: u32, name: &str, email: &str, car: Option<Car>) -> Self {
        Client {
            id,
            name: name.to_string(),
            email: email.to_string(),
            car,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn car(&self) -> Option<Car> {
        self.car.clone()
    }
}

impl Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ID: {} | Name: {} | Email: {})",
            self.name, self.id, self.email
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Employee {
    id: u32,
    name: String,
    role: Role,
}

impl Employee {
    pub fn new(id: u32, name: &str, role: Role) -> Self {
        Employee {
            id,
            name: name.to_string(),
            role,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn role(&self) -> Role {
        self.role
    }
}

impl Display for Employee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ID: {} | Name: {} | Role: {})",
            self.id, self.name, self.role
        )
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Order {
    id: u32,
    client_id: u32,
    service: Service,
    finished: bool,
}

impl Order {
    pub fn new(id: u32, client_id: u32, service: Service, finished: bool) -> Self {
        Order {
            id,
            client_id,
            service,
            finished,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn client_id(&self) -> u32 {
        self.client_id
    }

    pub fn service(&self) -> Service {
        self.service
    }

    pub fn finished(&self) -> bool {
        self.finished
    }
}

impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ID: {} | Client: {} | Service: {} | Finished: {}",
            self.id, self.client_id, self.service, self.finished
        )
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Report {
    id: u32,
    client_id: u32,
    order_id: u32,
    cost: u32,
}

impl Report {
    pub fn new(id: u32, client_id: u32, order_id: u32, cost: u32) -> Self {
        Report {
            id,
            client_id,
            order_id,
            cost,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn client_id(&self) -> u32 {
        self.client_id
    }

    pub fn order_id(&self) -> u32 {
        self.order_id
    }

    pub fn cost(&self) -> u32 {
        self.cost
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | Order: {} | Client: {} | Cost: ${}.{}",
            self.id,
            self.order_id,
            self.client_id,
            self.cost / 100,
            self.cost % 100
        )
    }
}
