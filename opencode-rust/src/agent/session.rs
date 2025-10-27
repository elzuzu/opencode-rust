use uuid::Uuid;

pub struct Session {
    id: Uuid,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}
