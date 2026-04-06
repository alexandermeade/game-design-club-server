use crate::*;

#[derive(Serialize)]
pub struct ProfileInfo {
    pub name: String,
    pub interests: Vec<String>
}

impl ProfileInfo {
    pub fn init(name: String, interests: Vec<String>) -> Self {
        return ProfileInfo { 
            name, 
            interests 
        }
    }
}


