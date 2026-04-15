#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    FromOnly,
    ToOnly,
    Both,
}

#[derive(Clone, Debug, Default)]
pub struct Language {
    pub code: String,
    pub name: String,
    pub size: String,
    pub direction: Direction,
    pub installed: bool,
    pub download_progress: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    NoLanguages = 0,
    Translation = 1,
    Settings = 2,
    ManageLanguages = 3,
}

impl Screen {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self::Both
    }
}

impl Default for Screen {
    fn default() -> Self {
        Self::NoLanguages
    }
}
