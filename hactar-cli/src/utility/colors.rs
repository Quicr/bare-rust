// Re-export colored for convenience
pub use colored::Colorize;

// Helper functions for consistent colored output
pub fn success(msg: &str) -> String {
    msg.bright_green().to_string()
}

pub fn error(msg: &str) -> String {
    msg.bright_red().to_string()
}

pub fn warning(msg: &str) -> String {
    msg.bright_yellow().to_string()
}

pub fn info(msg: &str) -> String {
    msg.bright_blue().to_string()
}

pub fn highlight(msg: &str) -> String {
    msg.bright_cyan().to_string()
}

pub fn emphasis(msg: &str) -> String {
    msg.bright_white().to_string()
}

pub fn dim(msg: &str) -> String {
    msg.white().to_string()
}
