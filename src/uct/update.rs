#[derive(Clone)]
pub struct Update {
    value: f32
}

impl Update {
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}
