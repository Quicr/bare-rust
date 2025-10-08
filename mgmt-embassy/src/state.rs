#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    #[allow(dead_code)]
    Normal,
    Debug,
}

impl Default for State {
    fn default() -> Self {
        State::Debug
    }
}

impl State {
    pub const fn new() -> Self {
        State::Debug
    }
}
