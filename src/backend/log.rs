// Copyright 2023 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Copy, Clone, Default)]
pub enum LogLevel {
    #[default]
    Debug,
    Info,
    Warning,
    Error,
}

pub trait Log {
    fn log(&self, level: LogLevel, text: String);

    fn flush(&self);
}
