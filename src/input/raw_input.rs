pub struct RawInput {
    pub name: String,
    pub value: f32,
}

impl RawInput {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: 0.0,
        }
    }
}
