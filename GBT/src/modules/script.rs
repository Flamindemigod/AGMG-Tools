use crate::{utils::exec_validation::Exectuable, CONFIG};
use file_diff::diff;
use log::{error, info, trace};
use std::io::{stderr, stdout, Write};
use std::{
    collections::VecDeque,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::exit,
    slice::Iter,
    str::FromStr,
};
use subprocess::{Popen, PopenConfig};

#[derive(Debug, Clone, Default)]
enum Operator {
    // Pipe Child Stdout into Next Child and Run New Child.
    // Pipe_Trigger, // |~|
    // Pipe Child Stdout into Next Child Stdin
    Pipe,
    // If Command Fail run Next
    Or,
    // If Command Pass run Next
    And,
    // Run Command and Terminate
    #[default]
    None,
}

impl ToString for Operator {
    fn to_string(&self) -> String {
        match self {
            Operator::Pipe => " | ".to_owned(),
            Operator::Or => " || ".to_owned(),
            Operator::And => " && ".to_owned(),
            Operator::None => "\n".to_owned(),
        }
    }
}

impl Operator {
    fn iter() -> Iter<'static, Operator> {
        static OPERATOR: [Operator; 4] =
            [Operator::Pipe, Operator::Or, Operator::And, Operator::None];
        OPERATOR.iter()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScriptParser {
    command: String,
    args: Vec<String>,
    operator: Operator,
    next_segment: Option<Box<ScriptParser>>,
}

impl FromStr for ScriptParser {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (op, index) = Operator::iter()
            .map(|op| (op.clone(), s.find(&op.to_string())))
            .fold(
                (Operator::None, Option::<usize>::None),
                |acc, (op, f)| match acc.1 {
                    Some(index) => match f {
                        Some(f) => {
                            if f < index {
                                return (op, Some(f));
                            } else {
                                return acc;
                            }
                        }
                        None => {
                            return acc;
                        }
                    },
                    None => (op, f),
                },
            );

        let split_string = match index {
            Some(slice_pos) => {
                let val = s.split_at(slice_pos);
                (val.0, val.1.split_once(op.to_string().as_str()).unwrap().1)
            }
            None => (s, ""),
        };
        let mut parser = Self::default();
        let mut splits: VecDeque<_> = split_string
            .0
            .split(" ")
            .map(|seg| seg.to_owned())
            .collect();
        if splits.len() > 0 {
            parser.command = splits.pop_front().unwrap();
            parser.args = splits.into();
            parser.operator = op.to_owned();
        }
        if !split_string.1.is_empty() {
            parser.next_segment = Some(Box::new(ScriptParser::from_str(split_string.1)?))
        }
        Ok(parser)
    }
}

pub trait Run {
    fn run(&self);
}

fn run_none(command: Vec<&str>) {
    trace!("Trying to run next command {:#?}", command);
    let mut p = Popen::create(
        &command,
        PopenConfig {
            // stdout: subprocess::Redirection::Pipe,
            ..Default::default()
        },
    )
    .unwrap();
    if let (Some(output), Some(output_error)) = (p.stdout.take(), p.stderr.take()) {
        let f_out = BufReader::new(output);
        let f_err = BufReader::new(output_error);
        let _ = stdout().lock().write_all(f_out.buffer());
        let _ = stderr().lock().write_all(f_err.buffer());
    }

    let exit_status = p.wait();
    match exit_status {
        Ok(status) => {
            if status.success() {
                trace!("process: {:#?}; Exited with {:#?}", command, status);
            }
        }
        Err(err) => {
            error!("Popen Failed on: {:#?}; Errored with {:#?}", command, err);
            exit(1)
        }
    }
}

fn run_and(command: Vec<&str>, segment: &ScriptParser) {
    let mut p = Popen::create(
        &command,
        PopenConfig {
            ..Default::default()
        },
    )
    .unwrap();
    if let (Some(output), Some(output_error)) = (p.stdout.take(), p.stderr.take()) {
        let f_out = BufReader::new(output);
        let f_err = BufReader::new(output_error);
        let _ = stdout().lock().write_all(f_out.buffer());
        let _ = stderr().lock().write_all(f_err.buffer());
    }

    let exit_status = p.wait();
    match exit_status {
        Ok(status) => {
            if status.success() {
                match &segment.next_segment {
                    Some(next_command) => {
                        next_command.run();
                    }
                    None => {
                        error!("process: {:#?}; Exited with {:#?}", command, status);
                        exit(1)
                    }
                }
            }
        }
        Err(err) => {
            error!("Popen Failed on: {:#?}; Errored with {:#?}", command, err);
            exit(1)
        }
    }
}

fn run_or(command: Vec<&str>, segment: &ScriptParser) {
    let mut p = Popen::create(
        &command,
        PopenConfig {
            ..Default::default()
        },
    )
    .unwrap();
    if let (Some(output), Some(output_error)) = (p.stdout.take(), p.stderr.take()) {
        let f_out = BufReader::new(output);
        let f_err = BufReader::new(output_error);
        let _ = stdout().lock().write_all(f_out.buffer());
        let _ = stderr().lock().write_all(f_err.buffer());
    }
    let exit_status = p.wait();
    match exit_status {
        Ok(status) => {
            if status.success() {
                info!("process: {:#?}; Exited with {:#?}", command, status);
            }
        }
        Err(err) => {
            error!("Popen Failed on: {:#?}; Errored with {:#?}", command, err);
            match &segment.next_segment {
                Some(next_command) => {
                    next_command.run();
                }
                None => (),
            }
        }
    }
}

impl Run for ScriptParser {
    fn run(&self) {
        let exectuable = Exectuable::new();
        let mut command = vec![if self.command == "$self" {
            if exectuable.eq(&CONFIG.lock().unwrap().execute) {
                exectuable.path.to_str().unwrap()
            } else {
                panic!("Executable Not Valid. Exiting");
            }
        } else {
            self.command.as_str()
        }];
        let mut args: Vec<_> = self.args.iter().map(|f| f.as_str()).collect();
        command.append(&mut args);
        match self.operator {
            Operator::And => run_and(command, self),
            Operator::None => run_none(command),
            Operator::Or => run_or(command, self),
            Operator::Pipe => {
                let mut p = Popen::create(
                    &command,
                    PopenConfig {
                        stdout: subprocess::Redirection::Pipe,
                        ..Default::default()
                    },
                )
                .unwrap();
                if let Some(output_error) = p.stderr.take() {
                    let f = BufReader::new(output_error);
                    let _ = stderr().lock().write_all(f.buffer());
                }

                if let Some(output) = &p.stdout {
                    let std_out = BufReader::new(output);

                    for line in std_out.lines() {
                        let val = serde_json::from_str::<PathBuf>(line.unwrap().as_str()).unwrap();
                        info!("New Line: {:#?}", val);
                        if diff(val.to_str().unwrap(), "./Config.yml") {
                            CONFIG.lock().unwrap().load_project_conf();
                        } else {
                            match &self.next_segment {
                                Some(next_command) => {
                                    next_command.run();
                                }
                                None => (),
                            }
                        }
                    }
                }
            }
        }
    }
}
