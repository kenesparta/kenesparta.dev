use uuid::Uuid;

pub struct PostUuid {}

impl PostUuid {
    pub fn new() -> String {
        Uuid::new_v4().to_string()
    }
}
