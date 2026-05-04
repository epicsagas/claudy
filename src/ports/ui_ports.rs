pub trait OutputPort {
    fn header(&mut self, title: &str);
    fn info(&mut self, msg: &str);
    fn success(&mut self, msg: &str);
    fn warn(&mut self, msg: &str);
    fn error(&mut self, msg: &str);
    fn write_line(&mut self, msg: &str) -> std::io::Result<()>;
}

pub trait PrompterPort {
    fn prompt(&mut self, label: &str, default_value: &str) -> anyhow::Result<String>;
    fn prompt_opt(&mut self, label: &str, default_value: &str) -> anyhow::Result<Option<String>>;
    fn prompt_secret(&mut self, label: &str) -> anyhow::Result<String>;
    fn prompt_secret_opt(&mut self, label: &str) -> anyhow::Result<Option<String>>;
    fn confirm(&mut self, label: &str, default_yes: bool) -> anyhow::Result<bool>;
    fn confirm_opt(&mut self, label: &str, default_yes: bool) -> anyhow::Result<Option<bool>>;
    fn select(&mut self, label: &str, items: &[String], default: usize) -> anyhow::Result<usize>;
    fn select_opt(
        &mut self,
        label: &str,
        items: &[String],
        default: usize,
    ) -> anyhow::Result<Option<usize>>;
}
