#[derive(Debug, Clone)]
pub enum UiEvent {
    Line(String),
    Success(String),
    Error(String),
}
