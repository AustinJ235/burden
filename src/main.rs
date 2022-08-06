// Copyright (c) 2022 Austin Johnson
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

#[macro_use]
extern crate crossterm;

use cargo_metadata::{CompilerMessage, Message};
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Print, Stylize};
use crossterm::terminal::{self, Clear, ClearType};
use std::env::args;
use std::io::{stdout, BufReader, Write};
use std::process::{Command, Stdio};

fn main() {
    let mut args = args();

    if args.len() == 0 {
        return display_help();
    }

    // Executable Path
    args.next().unwrap();

    let cmd = match args.next() {
        Some(cmd) => cmd,
        None => return display_help(),
    };

    match cmd.as_str() {
        "build" => (),
        "check" => (),
        "run" => (),
        "clippy" => (),
        "-h" | "--help" | "help" => {
            display_help();
            return;
        }
        _ => {
            println!("{}: no such subcommand: '{}'", "error".red(), cmd);
            return;
        }
    }

    let mut filtered_args: Vec<_> = Vec::new();
    let mut color = String::from("--color=always");

    for arg in args {
        if arg == "-h" || arg == "--help" {
            display_help();
            return;
        }

        if arg.starts_with("--color") {
            color = arg.to_string();
        }

        if !arg.starts_with("--message-format") {
            filtered_args.push(arg);
        }
    }

    let mut cmd_args = vec![
        cmd,
        color,
        "--message-format=json-diagnostic-rendered-ansi".to_string(),
    ];

    cmd_args.append(&mut filtered_args);
    let mut cmd = Command::new("cargo");
    cmd.stdout(Stdio::piped());
    cmd.args(cmd_args);
    let mut child = cmd.spawn().unwrap();
    let output = BufReader::new(child.stdout.take().unwrap());

    let messages: Vec<CompilerMessage> = Message::parse_stream(output)
        .filter_map(|msg_r| {
            if let Ok(Message::CompilerMessage(msg)) = msg_r {
                Some(msg)
            } else {
                None
            }
        })
        .collect();

    if messages.is_empty() {
        return;
    }

    terminal::enable_raw_mode().unwrap();
    execute!(stdout(), Clear(ClearType::All), cursor::Hide).unwrap();
    let mut displaying: usize = 0;
    let mut scroll: usize = 0;

    let help_line: String = [
        "\n".reset(),
        "Esc ".blue(),
        "Exit           ".green(),
        "Left ".blue(),
        "Prev Msg      ".green(),
        "Right ".blue(),
        "Next Msg     ".green(),
        "Up ".blue(),
        "Scroll Up       ".green(),
        "Down ".blue(),
        "Scroll Down   ".green(),
        "Home ".blue(),
        "First Msg     ".green(),
        "End ".blue(),
        "Last Msg       ".green(),
    ]
    .into_iter()
    .map(|content| format!("{}", content))
    .collect();

    let display_message = |index: usize, scroll: usize| {
        let term_h = terminal::size().unwrap().1.max(5);

        queue!(
            stdout(),
            cursor::MoveTo(0, 0),
            Clear(ClearType::All),
            Print(format!(
                "{} {} {} {}\n\n",
                "Displaying Message".green(),
                format!("{}", index + 1).blue(),
                "of".green(),
                format!("{}", messages.len()).blue(),
            )),
        )
        .unwrap();

        let msg_h = term_h - 4;
        let mut msg_lines = messages[index]
            .message
            .rendered
            .as_ref()
            .unwrap()
            .lines()
            .skip(scroll);

        for _ in 0..msg_h {
            queue!(
                stdout(),
                match msg_lines.next() {
                    Some(line) => Print(format!("{}\n", line)),
                    None => Print("\n".into()),
                }
            )
            .unwrap();
        }

        queue!(stdout(), Print(&help_line)).unwrap();
        stdout().flush().unwrap();
    };

    display_message(0, 0);

    while let Ok(event) = event::read() {
        if let Event::Key(KeyEvent { code, modifiers }) = event {
            if modifiers.contains(KeyModifiers::CONTROL) {
                if let KeyCode::Char('c') = code {
                    terminal::disable_raw_mode().unwrap();
                    execute!(stdout(), cursor::Show).unwrap();
                    return;
                }
            } else {
                match code {
                    KeyCode::Esc => {
                        terminal::disable_raw_mode().unwrap();
                        execute!(stdout(), cursor::Show).unwrap();
                        return;
                    },
                    KeyCode::Left if displaying > 0 => {
                        displaying -= 1;
                        scroll = 0;
                        display_message(displaying, scroll);
                    }
                    KeyCode::Right if displaying < messages.len() - 1 => {
                        displaying += 1;
                        scroll = 0;
                        display_message(displaying, scroll);
                    }
                    KeyCode::Up => {
                        if scroll > 0 {
                            scroll -= 1;
                            display_message(displaying, scroll);
                        }
                    }
                    KeyCode::Down => {
                        scroll += 1;
                        display_message(displaying, scroll);
                    }
                    KeyCode::Home => {
                        displaying = 0;
                        scroll = 0;
                        display_message(displaying, scroll);
                    }
                    KeyCode::End => {
                        displaying = messages.len() - 1;
                        scroll = 0;
                        display_message(displaying, scroll);
                    }
                    _ => (),
                }
            }
        }
    }

    child.wait().unwrap();
}

fn display_help() {
    println!(
        r#"
Error/Warning Pager for Cargo

USAGE:
    burden [SUBCOMMAND] [SUBCOMMAND OPTIONS]

Supported cargo commands are:
    build    Compile the current package
    check    Analyze the current package and report errors, but don't build object files
    run      Run a binary or example of the local package
    clippy   Checks a package to catch common mistakes and improve your Rust code."#
    );
}
