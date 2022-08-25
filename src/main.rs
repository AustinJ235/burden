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
use crossterm::event::KeyEventKind;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Print, Stylize};
use crossterm::terminal::{self, Clear, ClearType};
use std::env::args;
use std::io::{self, stdout, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn main() {
    let mut args = args();

    if args.len() == 0 {
        return display_help();
    }

    // Executable Path
    args.next().unwrap();
    let mut subcommand: Option<String> = None;
    let mut working_dir = String::from(".");
    let mut color = String::from("always");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-V" | "--version" => {
                print!("burden 0.2.0 (2022-08-25) / ");
                let cargo_out = Command::new("cargo").arg("-V").output().unwrap();
                stdout().write_all(&cargo_out.stdout).unwrap();
                return;
            }
            "--working-dir" => match args.next() {
                Some(dir) => working_dir = dir,
                None => {
                    println!("{}: The argument '--working-dir <DIRECTORY>' requires a value but none was supplied", "error".red());
                    return;
                }
            },
            "--color" => {
                match args.next() {
                    Some(when) => match when.as_str() {
                        "auto" | "always" | "never" => color = when,
                        _ => {
                            println!("{}: The argument '--color <WHEN>' requires WHEN to be 'auto', 'always', or 'never'.", "error".red());
                            return;
                        }
                    },
                    None => {
                        println!("{}: The argument '--color <WHEN>' requires a value but none was supplied", "error".red());
                        return;
                    }
                }
            }
            "--help" | "-h" => {
                display_help();
                return;
            }
            "build" | "b" | "check" | "c" | "run" | "r" | "clippy" => {
                subcommand = Some(arg);
                break;
            }
            _ => {
                if arg.starts_with("--working-dir=") {
                    let dir = arg.split('=').nth(1).unwrap();

                    if dir.is_empty() {
                        println!("{}: The argument '--working-dir=<DIRECTORY>' requires a value but none was supplied", "error".red());
                        return;
                    }

                    working_dir = dir.to_string();
                } else if arg.starts_with("--color=") {
                    let when = arg.split('=').nth(1).unwrap();

                    if when.is_empty() {
                        println!("{}: The argument '--color=<WHEN>' requires a value but none was supplied", "error".red());
                        return;
                    }

                    match when {
                        "auto" | "always" | "never" => color = when.to_string(),
                        _ => {
                            println!("{}: The argument '--color=<WHEN>' requires WHEN to be 'auto', 'always', or 'never'.", "error".red());
                            return;
                        }
                    }
                } else {
                    println!("{}: Found argument '{}' which wasn't expected, or isn't valid in this context", "error".red(), arg);
                    return;
                }
            }
        }
    }

    let subcommand = match subcommand {
        Some(some) => some,
        None => {
            display_help();
            return;
        }
    };

    let mut subcommand_args: Vec<String> = match color.as_str() {
        "always" | "auto" => vec![
            "--message-format=json-diagnostic-rendered-ansi".to_string(),
            "--color=always".to_string(),
        ],
        _ => vec![
            "--message-format=json".to_string(),
            "--color=never".to_string(),
        ],
    };

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--color" => match args.next() {
                Some(_) => println!(
                    "{}: Found argument '--color <WHEN>' which is overrided by burden.",
                    "warning".dark_yellow()
                ),
                None => break,
            },
            "--message-format" => match args.next() {
                Some(_) => println!(
                    "{}: Found argument '--message-format <FMT>' which is overrided by burden.",
                    "warning".dark_yellow()
                ),
                None => break,
            },
            "--help" | "-h" => {
                let cargo_out = Command::new("cargo")
                    .arg(subcommand)
                    .arg(arg)
                    .output()
                    .unwrap();
                stdout().write_all(&cargo_out.stdout).unwrap();
                return;
            }
            _ => {
                if arg.starts_with("--color=") {
                    println!(
                        "{}: Found argument '--color=<WHEN>' which is overrided by burden.",
                        "warning".dark_yellow()
                    );
                } else if arg.starts_with("--message-format=") {
                    println!(
                        "{}: Found argument '--message-format=<FMT>' which is overrided by burden.",
                        "warning".dark_yellow()
                    );
                } else {
                    subcommand_args.push(arg);
                }
            }
        }
    }

    let is_run_cmd = matches!(subcommand.as_str(), "run" | "r");
    let mut cmd = Command::new("cargo");
    cmd.stdout(Stdio::piped());
    cmd.current_dir(working_dir);
    cmd.arg(format!("--color={}", color));
    cmd.arg(subcommand);
    cmd.args(subcommand_args);

    let mut child = cmd.spawn().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut messages: Vec<CompilerMessage> = Vec::new();

    for message in Message::parse_stream(&mut output).flatten() {
        match message {
            Message::CompilerMessage(compiler_msg) if compiler_msg.message.code.is_some() => {
                messages.push(compiler_msg)
            }
            Message::BuildFinished(_) => {
                thread::sleep(Duration::from_millis(100));
                break;
            }
            _ => (),
        }
    }

    if messages.is_empty() {
        if is_run_cmd {
            thread::spawn(move || {
                let _ = io::copy(&mut output, &mut stdout());
            });
        }

        child.wait().unwrap();
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
        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event
        {
            if modifiers.contains(KeyModifiers::CONTROL) {
                if let KeyCode::Char('c') = code {
                    terminal::disable_raw_mode().unwrap();
                    execute!(stdout(), cursor::Show).unwrap();
                    break;
                }
            } else {
                match code {
                    KeyCode::Esc => {
                        terminal::disable_raw_mode().unwrap();
                        execute!(stdout(), cursor::Show).unwrap();
                        break;
                    }
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

    if is_run_cmd {
        thread::spawn(move || {
            let _ = io::copy(&mut output, &mut stdout());
        });
    }

    child.wait().unwrap();
}

fn display_help() {
    println!(
        r#"
Error/Warning Pager for Cargo

USAGE:
    burden [SUBCOMMAND] [SUBCOMMAND OPTIONS]

OPTIONS:
    -V, --version                    Print version info and exit
        --color <WHEN>               Coloring: auto, always, never
        --working-dir <DIRECTORY>    Directory to run cargo in
    -h, --help                       Print help information

Supported cargo commands are:
    build    Compile the current package
    check    Analyze the current package and report errors, but don't build object files
    run      Run a binary or example of the local package
    clippy   Checks a package to catch common mistakes and improve your Rust code."#
    );
}
