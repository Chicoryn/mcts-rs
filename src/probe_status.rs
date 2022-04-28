#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProbeStatus {
    Busy,
    Empty,
    Existing(usize),
    Expanded
}
