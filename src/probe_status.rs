#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProbeStatus {
    Success,
    NoChildren,
    AlreadyExpanded(usize),
    AlreadyExpanding,
}
