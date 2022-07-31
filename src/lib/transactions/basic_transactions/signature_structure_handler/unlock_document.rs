use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct UnlockDocument {}

impl UnlockDocument {
    pub fn get_null() -> UnlockDocument {
        return UnlockDocument {};
    }
}