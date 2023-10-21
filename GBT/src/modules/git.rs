use std::{
    collections::VecDeque,
    ffi::OsStr,
    fmt::{Display, Write},
    fs::{self, canonicalize},
    path::PathBuf,
};

use anyhow::{Result, Error};
use indicatif::{MultiProgress, ProgressBar, ProgressState, ProgressStyle};
use log::{error, info, trace};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::utils::download::Downloader;

#[derive(Clone, Debug, Default)]
pub struct Git {
    owner: String,
    repo: String,
    path: String,
}

fn match_seg(segment: &OsStr, pattern: &str) -> bool {
    let pattern_match = segment.to_string_lossy().contains(pattern);
    if pattern_match {
        trace!("Matched {segment:#?} to {pattern:#?}");
    }
    pattern_match
}

impl Git {
    pub fn load(&mut self, url: &PathBuf) -> Result<Self> {
        let mut split_path = url.iter().collect::<VecDeque<_>>();
        while !split_path.is_empty() {
            let path_seg = split_path.pop_front().unwrap();
            if match_seg(path_seg, "github") {
                self.owner = split_path
                    .pop_front()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                self.repo = split_path
                    .pop_front()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                split_path.pop_front();
                split_path.pop_front();
                self.path = split_path
                    .iter()
                    .map(|f| f.to_str().unwrap().to_string())
                    .collect::<Vec<String>>()
                    .join("/");
            }
        }
        Ok(self.to_owned())
    }
    fn to_api(&self) -> Result<String> {
        Ok(format!(
            "https://api.github.com/repos/{:}/{:}/contents/{:}",
            self.owner, self.repo, self.path
        )
        .to_string())
    }

    pub fn download(&self, target_dir: PathBuf) -> Result<Vec<PathBuf>> {
        let response = minreq::get(self.to_api()?).with_header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0. 4389.82 Safari/537.36").send()?.json::<Vec<RepoItem>>()?;
        let multi = MultiProgress::new();
        let results: Result<Vec<_>, _> = response
            .par_iter()
            .map(|item| item.download(&multi, &target_dir))
            .collect();
        
        let parsed_res = results?;
        let _: Vec<()> = parsed_res.clone().par_iter().map(|res| info!("Downloaded : {:#?}", res)).collect();
        trace!("Download Finished");
        Ok(parsed_res)
    }
}

impl Display for Git {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Repo Owner: {:?}\nRepo: {:?}\nPath: {:?}\n",
            self.owner, self.repo, self.path
        )
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct RepoItem {
    pub name: String,
    pub sha: String,
    pub download_url: String,
}

impl RepoItem {
    pub fn download(&self, multi: &MultiProgress, target: &PathBuf) -> Result<PathBuf> {
        fs::create_dir_all(target)?;
        let file_name = self.name.clone();
        let target_path = target.clone().join(&file_name);
        trace!("Downloading to {:#?}", target_path);
        let pb = multi.add(ProgressBar::new(100));
        let sty = ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] {wide_bar:.cyan/blue} {bytes}/{total_bytes} {msg} {eta}",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("##-");
        pb.set_style(sty);
        pb.set_message(format!("{:}", file_name));
        Downloader::new(self.download_url.as_str())?.download(
            target_path.clone(),
            move |downloaded, size| {
                pb.set_length(size);
                pb.set_position(downloaded);
                // pb.with_finish(indicatif::ProgressFinish::AndClear);
            },
        )?;
        Ok(target_path)
    }
}
