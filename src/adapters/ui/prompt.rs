use dialoguer::{Input, Password, Select, theme::ColorfulTheme};
use std::io;

use crate::ports::ui_ports::PrompterPort;

pub struct Prompter {
    _reader: std::io::BufReader<io::Stdin>,
}

impl PrompterPort for Prompter {
    fn prompt(&mut self, label: &str, default_value: &str) -> anyhow::Result<String> {
        let theme = ColorfulTheme::default();
        if !default_value.is_empty() {
            Ok(Input::<String>::with_theme(&theme)
                .with_prompt(label)
                .default(default_value.to_string())
                .interact_text()?)
        } else {
            Ok(Input::<String>::with_theme(&theme)
                .with_prompt(label)
                .interact_text()?)
        }
    }

    fn prompt_opt(&mut self, label: &str, default_value: &str) -> anyhow::Result<Option<String>> {
        let theme = ColorfulTheme::default();
        let input = Input::<String>::with_theme(&theme);
        let input = input.with_prompt(label).allow_empty(true);
        let res = if !default_value.is_empty() {
            input.default(default_value.to_string()).interact_text()
        } else {
            input.interact_text()
        };

        match res {
            Ok(value) => Ok(Some(value)),
            Err(dialoguer::Error::IO(io_err))
                if io_err.kind() == io::ErrorKind::Interrupted
                    || io_err.kind() == io::ErrorKind::UnexpectedEof =>
            {
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    fn prompt_secret(&mut self, label: &str) -> anyhow::Result<String> {
        let theme = ColorfulTheme::default();
        Ok(Password::with_theme(&theme).with_prompt(label).interact()?)
    }

    fn prompt_secret_opt(&mut self, label: &str) -> anyhow::Result<Option<String>> {
        let theme = ColorfulTheme::default();
        let res = Password::with_theme(&theme)
            .with_prompt(label)
            .allow_empty_password(true)
            .interact();

        match res {
            Ok(value) => Ok(Some(value)),
            Err(dialoguer::Error::IO(io_err))
                if io_err.kind() == io::ErrorKind::Interrupted
                    || io_err.kind() == io::ErrorKind::UnexpectedEof =>
            {
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    fn confirm(&mut self, label: &str, default_yes: bool) -> anyhow::Result<bool> {
        let theme = ColorfulTheme::default();
        Ok(dialoguer::Confirm::with_theme(&theme)
            .with_prompt(label)
            .default(default_yes)
            .interact()?)
    }

    fn confirm_opt(&mut self, label: &str, default_yes: bool) -> anyhow::Result<Option<bool>> {
        let theme = ColorfulTheme::default();
        match dialoguer::Confirm::with_theme(&theme)
            .with_prompt(label)
            .default(default_yes)
            .interact_opt()
        {
            Ok(val) => Ok(val),
            Err(dialoguer::Error::IO(io_err))
                if io_err.kind() == io::ErrorKind::Interrupted
                    || io_err.kind() == io::ErrorKind::UnexpectedEof =>
            {
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    fn select(&mut self, label: &str, items: &[String], default: usize) -> anyhow::Result<usize> {
        let theme = ColorfulTheme::default();
        Ok(Select::with_theme(&theme)
            .with_prompt(label)
            .items(items)
            .default(default)
            .max_length(10)
            .interact()?)
    }

    fn select_opt(
        &mut self,
        label: &str,
        items: &[String],
        default: usize,
    ) -> anyhow::Result<Option<usize>> {
        let theme = ColorfulTheme::default();
        Ok(Select::with_theme(&theme)
            .with_prompt(label)
            .items(items)
            .default(default)
            .max_length(10)
            .interact_opt()?)
    }
}

impl Prompter {
    pub fn new(_stdin: io::Stdin, _stdout: io::Stdout) -> Self {
        Prompter {
            _reader: std::io::BufReader::new(io::stdin()),
        }
    }
}
