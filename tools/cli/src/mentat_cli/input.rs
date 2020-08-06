// Copyright 2017 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use std::io::{stdin, stdout, Write};

use linefeed::{DefaultTerminal, Interface, ReadResult, Signal};

use termion::color;

use self::InputResult::*;

use command_parser::{command, Command};

use failure::Error;

/// Starting prompt
const DEFAULT_PROMPT: &str = "mentat=> ";
/// Prompt when further input is being read
// TODO: Should this actually reflect the current open brace?
const MORE_PROMPT: &str = "mentat.> ";

/// Possible results from reading input from `InputReader`
#[derive(Clone, Debug)]
pub enum InputResult {
    /// mentat command as input; (name, rest of line)
    MetaCommand(Command),
    /// An empty line
    Empty,
    /// Needs more input
    More,
    /// End of file reached
    Eof,
}

/// Reads input from `stdin`
pub struct InputReader {
    buffer: String,
    interface: Option<Interface<DefaultTerminal>>,
    in_process_cmd: Option<Command>,
}

enum UserAction {
    // We've received some text that we should interpret as a new command, or
    // as part of the current command.
    TextInput(String),
    // We were interrupted, if we have a current command we should clear it,
    // otherwise we should exit. Currently can only be generated by reading from
    // a terminal (and not by reading from stdin).
    Interrupt,
    // We hit the end of the file, there was an error getting user input, or
    // something else happened that means we should exit.
    Quit,
}

impl InputReader {
    /// Constructs a new `InputReader` reading from `stdin`.
    pub fn new(interface: Option<Interface<DefaultTerminal>>) -> InputReader {
        if let Some(ref interface) = interface {
            // It's fine to fail to load history.
            let p = ::history_file_path();
            let loaded = interface.load_history(&p);
            debug!("history read from {}: {}", p.display(), loaded.is_ok());

            let mut r = interface.lock_reader();
            // Handle SIGINT (Ctrl-C)
            r.set_report_signal(Signal::Interrupt, true);
            r.set_word_break_chars(" \t\n!\"#$%&'(){}*+,-./:;<=>?@[\\]^`");
        }

        InputReader {
            buffer: String::new(),
            interface,
            in_process_cmd: None,
        }
    }

    /// Returns whether the `InputReader` is reading from a TTY.
    pub fn is_tty(&self) -> bool {
        self.interface.is_some()
    }

    /// Reads a single command, item, or statement from `stdin`.
    /// Returns `More` if further input is required for a complete result.
    /// In this case, the input received so far is buffered internally.
    pub fn read_input(&mut self) -> Result<InputResult, Error> {
        let prompt = if self.in_process_cmd.is_some() {
            MORE_PROMPT
        } else {
            DEFAULT_PROMPT
        };
        let prompt = format!(
            "{blue}{prompt}{reset}",
            blue = color::Fg(::BLUE),
            prompt = prompt,
            reset = color::Fg(color::Reset)
        );
        let line = match self.read_line(prompt.as_str()) {
            UserAction::TextInput(s) => s,
            UserAction::Interrupt if self.in_process_cmd.is_some() => {
                self.in_process_cmd = None;
                self.buffer.clear();
                // Move to the next line, so that our next prompt isn't on top
                // of the previous.
                println!();
                String::new()
            }
            _ => return Ok(Eof),
        };

        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }

        self.buffer.push_str(&line);

        if self.buffer.is_empty() {
            return Ok(Empty);
        }

        // if we have a command in process (i.e. an incomplete query or transaction),
        // then we already know which type of command it is and so we don't need to parse the
        // command again, only the content, which we do later.
        // Therefore, we add the newly read in line to the existing command args.
        // If there is no in process command, we parse the read in line as a new command.
        let cmd = match &self.in_process_cmd {
            &Some(Command::QueryPrepared(ref args)) => {
                Ok(Command::QueryPrepared(args.clone() + "\n" + &line))
            }
            &Some(Command::Query(ref args)) => Ok(Command::Query(args.clone() + "\n" + &line)),
            &Some(Command::Transact(ref args)) => {
                Ok(Command::Transact(args.clone() + "\n" + &line))
            }
            _ => command(&self.buffer),
        };

        match cmd {
            Ok(cmd) => {
                match cmd {
                    Command::Query(_)
                    | Command::QueryPrepared(_)
                    | Command::Transact(_)
                    | Command::QueryExplain(_)
                        if !cmd.is_complete() =>
                    {
                        // A query or transact is complete if it contains a valid EDN.
                        // if the command is not complete, ask for more from the REPL and remember
                        // which type of command we've found here.
                        self.in_process_cmd = Some(cmd);
                        Ok(More)
                    }
                    _ => {
                        let entry = self.buffer.clone();
                        self.buffer.clear();
                        self.add_history(entry);
                        self.in_process_cmd = None;
                        Ok(InputResult::MetaCommand(cmd))
                    }
                }
            }
            Err(e) => {
                let entry = self.buffer.clone();
                self.buffer.clear();
                self.add_history(entry);
                self.in_process_cmd = None;
                Err(e)
            }
        }
    }

    fn read_line(&mut self, prompt: &str) -> UserAction {
        match self.interface {
            Some(ref mut r) => {
                r.set_prompt(prompt).unwrap();
                r.read_line()
                    .ok()
                    .map_or(UserAction::Quit, |line| match line {
                        ReadResult::Input(s) => UserAction::TextInput(s),
                        ReadResult::Signal(Signal::Interrupt) => UserAction::Interrupt,
                        _ => UserAction::Quit,
                    })
            }
            None => {
                print!("{}", prompt);
                if stdout().flush().is_err() {
                    return UserAction::Quit;
                }
                self.read_stdin()
            }
        }
    }

    fn read_stdin(&self) -> UserAction {
        let mut s = String::new();

        match stdin().read_line(&mut s) {
            Ok(0) | Err(_) => UserAction::Quit,
            Ok(_) => {
                if s.ends_with("\n") {
                    let len = s.len() - 1;
                    s.truncate(len);
                }
                UserAction::TextInput(s)
            }
        }
    }

    fn add_history(&self, line: String) {
        if let Some(ref interface) = self.interface {
            interface.add_history(line);
        }
        self.save_history();
    }

    pub fn save_history(&self) {
        if let Some(ref interface) = self.interface {
            let p = ::history_file_path();
            // It's okay to fail to save history.
            let saved = interface.save_history(&p);
            debug!("history saved to {}: {}", p.display(), saved.is_ok());
        }
    }
}
