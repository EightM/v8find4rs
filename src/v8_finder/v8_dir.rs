use std::hash::{Hash, Hasher};
use std::path::PathBuf;


#[derive(Debug, Clone)]
pub struct V8Dir {
    pub path: PathBuf,
}

impl V8Dir {
    pub fn from_path(path: PathBuf) -> Self {
        V8Dir {
            path,
        }
    }
}

impl PartialEq for V8Dir {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl Eq for V8Dir {}

impl Hash for V8Dir {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state)
    }
}