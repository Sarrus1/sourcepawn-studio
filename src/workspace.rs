use crate::environment::Environment;

#[derive(Debug, Clone, Default)]
pub struct Workspace {
    pub environment: Environment,
}

impl Workspace {
    pub fn new(environment: Environment) -> Self {
        Self {
            environment,
            ..Self::default()
        }
    }
}
