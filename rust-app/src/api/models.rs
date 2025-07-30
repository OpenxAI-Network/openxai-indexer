use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claimer {
    pub account: String,
}
