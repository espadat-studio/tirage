use std::io::{self, IsTerminal};
use std::str::FromStr;

use dialoguer::console::Term;
use dialoguer::theme::{ColorfulTheme, SimpleTheme, Theme};
use dialoguer::{Confirm, FuzzySelect, Input};

use crate::Failure;

pub struct Choice {
    pub noun: &'static str,
    pub prompt: &'static str,
    pub resolved_by: &'static str,
}

pub struct Prompt {
    subcommand: &'static str,
    interactive: bool,
    theme: Box<dyn Theme>,
}

impl Prompt {
    pub fn new(subcommand: &'static str, no_input: bool, colour: bool) -> Self {
        let interactive = io::stdin().is_terminal()
            && io::stderr().is_terminal()
            && !no_input
            && std::env::var_os("TIRAGE_NO_INPUT").is_none_or(|value| value.is_empty())
            && std::env::var_os("TERM").is_none_or(|term| term != "dumb");
        let theme: Box<dyn Theme> = if colour {
            Box::new(ColorfulTheme::default())
        } else {
            Box::new(SimpleTheme)
        };
        Self {
            subcommand,
            interactive,
            theme,
        }
    }

    pub fn is_interactive(&self) -> bool {
        self.interactive
    }

    pub fn select<T: Clone>(
        &self,
        items: &[T],
        display: impl Fn(&T) -> String,
        choice: &Choice,
    ) -> Result<T, Failure> {
        self.check(choice)?;
        let rows: Vec<String> = items.iter().map(display).collect();
        let picked = FuzzySelect::with_theme(self.theme.as_ref())
            .with_prompt(choice.prompt)
            .items(&rows)
            .default(0)
            .interact_opt();
        match picked {
            Ok(Some(index)) => Ok(items[index].clone()),
            _ => Err(aborted(choice)),
        }
    }

    pub fn ask<T>(
        &self,
        choice: &Choice,
        default: Option<T>,
        validate: impl Fn(&T) -> Result<(), String>,
    ) -> Result<T, Failure>
    where
        T: Clone + ToString + FromStr,
        T::Err: ToString,
    {
        self.check(choice)?;
        let mut input = Input::with_theme(self.theme.as_ref())
            .with_prompt(choice.prompt)
            .validate_with(|value: &T| validate(value));
        if let Some(default) = default {
            input = input.default(default);
        }
        input.interact_text().map_err(|_| aborted(choice))
    }

    pub fn confirm(&self, prompt: &str, choice: &Choice) -> Result<bool, Failure> {
        self.check(choice)?;
        match Confirm::with_theme(self.theme.as_ref())
            .with_prompt(prompt)
            .default(false)
            .interact_opt()
        {
            Ok(Some(answer)) => Ok(answer),
            _ => Err(aborted(choice)),
        }
    }

    pub fn check(&self, choice: &Choice) -> Result<(), Failure> {
        if self.interactive {
            return Ok(());
        }
        Err(Failure::Usage(
            self.subcommand,
            format!("no {} given, pass {}", choice.noun, choice.resolved_by),
            None,
        ))
    }
}

fn aborted(choice: &Choice) -> Failure {
    let _ = Term::stderr().show_cursor();
    Failure::Aborted(format!("no {} selected", choice.noun))
}
