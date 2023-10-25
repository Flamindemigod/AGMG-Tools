// use crate::CONFIG;
// use file_diff::diff;
// use std::io::prelude::*;
// use std::{io::BufReader, path::PathBuf};
// use subprocess::{Popen, PopenConfig, Redirection};

// use super::exec_validation::Exectuable;

use crate::modules::script::{ScriptParser, Run};
use std::str::FromStr;


pub fn test() {
    let script = ScriptParser::from_str("$self -ddd watch | $self -ddd build").unwrap();
    script.run();
}
