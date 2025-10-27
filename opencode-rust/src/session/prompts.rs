pub struct SessionPrompts;

impl SessionPrompts {
    pub fn plan_reminder() -> &'static str {
        include_str!("prompts/plan.txt")
    }

    pub fn build_switch() -> &'static str {
        include_str!("prompts/build-switch.txt")
    }
}
