#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Running,
    Normal,
    Debug,
}

pub const DEFAULT_STATE: State = State::Debug;
