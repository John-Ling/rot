#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    LEFT,
    RIGHT,
    INCREMENT,
    DECREMENT,
    OUTPUT,
    INPUT,
    JUMPFWD,
    JUMPBCK,
}
