use sourcepawn_lexer::{TextRange, TextSize};

/// State of a preprocessor condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionState {
    /// The condition is not activated and could be activated by an else/elseif directive.
    NotActivated,

    /// The condition has been activated, all related else/elseif directives should be skipped.
    Activated,

    /// The condition is active and the preprocessor should process the code.
    Active,
}

#[derive(Debug, Default)]
pub struct ConditionStack {
    stack: Vec<ConditionState>,
}

impl ConditionStack {
    pub fn top(&self) -> Option<&ConditionState> {
        self.stack.last()
    }

    pub fn pop(&mut self) -> Option<ConditionState> {
        self.stack.pop()
    }

    pub fn push(&mut self, condition: ConditionState) {
        self.stack.push(condition);
    }

    pub fn top_is_activated_or_not_activated(&self) -> bool {
        if let Some(top) = self.top() {
            matches!(
                top,
                ConditionState::Activated | ConditionState::NotActivated
            )
        } else {
            false
        }
    }
}

#[derive(Debug, Default)]
pub struct ConditionOffsetStack {
    stack: Vec<TextSize>,
    skipped_ranges: Vec<TextRange>,
}

impl ConditionOffsetStack {
    pub fn top(&self) -> Option<&TextSize> {
        self.stack.last()
    }

    pub fn pop(&mut self) -> Option<TextSize> {
        self.stack.pop()
    }

    pub fn pop_and_push_skipped_range(&mut self, end: TextSize) {
        if let Some(start) = self.pop() {
            self.push_skipped_range(TextRange::new(start, end));
        }
    }

    pub fn push_skipped_range(&mut self, range: TextRange) {
        self.skipped_ranges.push(range);
    }

    pub fn push(&mut self, offset: TextSize) {
        self.stack.push(offset);
    }

    pub fn skipped_ranges(&self) -> &[TextRange] {
        &self.skipped_ranges
    }

    pub fn sort_skipped_ranges(&mut self) {
        self.skipped_ranges.sort_by(|a, b| a.ordering(*b));
    }
}
