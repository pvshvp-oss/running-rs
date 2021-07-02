#![cfg(feature = "logging")]

// IMPORTS

use log::log;
use log::Level;
use std::borrow::Cow;

// STRUCTS

#[derive(Debug, Clone)]
pub struct LoggingPreferences<'a> {
    label: Cow<'a, str>,
    entry_level: Option<LogEntryLevel>,
}

#[derive(Debug, Clone)]
pub struct LoggingData<'b, 'c> {
    pub input: Cow<'b, str>,
    pub output: Cow<'c, str>,
}

#[derive(Debug, Copy, Clone)]
pub struct LogEntryLevel {
    pub input_level: Option<Level>,
    pub output_level: Option<Level>,
}

// STRUCT IMPLEMENTATIONS

impl<'a> LoggingPreferences<'a> {
    pub fn new<S: Into<Cow<'a, str>>>(label: S) -> Self {
        let mut logging_preferences = LoggingPreferences::default();
        logging_preferences.label = label.into();
        logging_preferences
    }

    // GETTERS

    pub fn label(&self) -> &str {
        self.label.as_ref()
    }

    pub fn entry_level(&self) -> &Option<LogEntryLevel> {
        &self.entry_level
    }

    // SETTERS

    pub fn set_label<S: Into<Cow<'a, str>>>(&mut self, label: S) {
        self.label = label.into();
    }

    pub fn set_file_level(&mut self, file_level: LogEntryLevel) {
        self.entry_level = Some(file_level);
    }
}

impl<'b, 'c> LoggingData<'b, 'c> {
    pub fn new() -> Self {
        LoggingData {
            input: "".into(),
            output: "".into(),
        }
    }

    // GETTERS

    pub fn input(&self) -> &str {
        self.input.as_ref()
    }

    pub fn output(&self) -> &str {
        self.output.as_ref()
    }

    // SETTERS

    pub fn set_input<S: Into<Cow<'b, str>>>(&mut self, input: S) {
        self.input = input.into();
    }

    pub fn set_output<S: Into<Cow<'c, str>>>(&mut self, output: S) {
        self.output = output.into();
    }
}

// TRAIT IMPLEMENTATIONS

impl<'a> Default for LoggingPreferences<'a> {
    fn default() -> Self {
        LoggingPreferences {
            label: "".into(),
            entry_level: Some(LogEntryLevel {
                input_level: Some(Level::Debug),
                output_level: Some(Level::Debug),
            }),
        }
    }
}

impl<'a, 'b, 'c, T: Loggable<'a, 'b, 'c>> From<&T> for LoggingPreferences<'a> {
    fn from(loggable_task: &T) -> Self {
        loggable_task.logging_preferences().clone()
    }
}

impl<'a> From<&LoggingPreferences<'a>> for LoggingPreferences<'a> {
    fn from(logging_preferences: &LoggingPreferences<'a>) -> Self {
        logging_preferences.clone()
    }
}

// TRAITS

pub trait Loggable<'a, 'b, 'c> {
    fn logging_preferences(&self) -> &LoggingPreferences<'a>;

    fn logging_data(&self) -> &LoggingData<'b, 'c>;

    fn input_prefix(&self) -> &str {
        ""
    }

    fn output_prefix(&self) -> &str {
        ""
    }

    fn log_input(&self) -> () {
        let logging_preferences = self.logging_preferences();
        if let Some(entry_level) = logging_preferences.entry_level() {
            if let Some(input_level) = entry_level.input_level {
                log!(
                    target: logging_preferences.label.as_ref(),
                    input_level,
                    "{}{}",
                    self.input_prefix(),
                    self.logging_data().input
                )
            }
        }
    }

    fn log_output(&self) -> () {
        let logging_preferences = self.logging_preferences();
        if let Some(entry_level) = logging_preferences.entry_level() {
            if let Some(output_level) = entry_level.output_level {
                log!(
                    target: logging_preferences.label.as_ref(),
                    output_level,
                    "{}{}",
                    self.output_prefix(),
                    self.logging_data().output
                )
            }
        }
    }

    fn log(&self) {
        self.log_input();
        self.log_output();
    }
}

#[derive(Debug, Clone)]
pub struct LoggingHandler {
    
}
