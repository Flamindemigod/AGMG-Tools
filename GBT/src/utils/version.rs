use std::fmt::{Debug, Display, Formatter, Write};

use anyhow::Result;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use log::info;
use serde_json::Value;

use super::{download::Downloader, exec_validation::Exectuable};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    pub version: [u8; 3],
}

impl Version {
    #[inline]
    pub fn new(a: u8, b: u8, c: u8) -> Self {
        Self { version: [a, b, c] }
    }

    pub fn from_str<T: AsRef<str>>(str: T) -> Option<Self> {
        let parts = str.as_ref().split('.').collect::<Vec<&str>>();

        if parts.len() != 3 {
            return None;
        }

        if let (Ok(a), Ok(b), Ok(c)) = (parts[0].parse(), parts[1].parse(), parts[2].parse()) {
            return Some(Version::new(a, b, c));
        }

        None
    }

    pub fn to_plain_string(&self) -> String {
        format!("{}{}{}", self.version[0], self.version[1], self.version[2])
    }

    pub fn has_update(&self) -> Result<impl Fn() -> Result<()>> {
        let data = minreq::get("https://api.github.com/repos/Flamindemigod/AGMG-Tools/releases/latest").with_header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0. 4389.82 Safari/537.36").send()?.json::<Value>()?;
        let most_recent = Self::from_str(
            data["tag_name"]
                .as_str()
                .unwrap()
                .strip_prefix("v")
                .unwrap(),
        )
        .unwrap();
        let has_update = most_recent.gt(self);
        let update = move || {
            if has_update {
                info!("GBT is up to date");
                return Ok(());
            }
            info!("A new version of GBT is available. Updating");
            let download_uri = data["assets"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|asset| asset["name"].as_str().unwrap() == "gbt.exe")
                .map(|asset| asset["browser_download_url"].as_str().unwrap())
                .find(|_| true)
                .unwrap();
            let pb = ProgressBar::new(100);
            let sty = ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] {wide_bar:.cyan/blue} {bytes}/{total_bytes} {msg} {eta}",
            )
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("##-");
            pb.set_style(sty);

            Downloader::new(download_uri).unwrap().download(
                Exectuable::default().path,
                move |downloaded, size| {
                    pb.set_length(size);
                    pb.set_position(downloaded);
                },
            )?;
            info!("Update Complete.");
            Ok(())
        };

        Ok(update)
    }

    pub fn update(&self) -> Result<bool> {
        let data = minreq::get("https://api.github.com/repos/Flamindemigod/AGMG-Tools/releases/latest").with_header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0. 4389.82 Safari/537.36").send()?.json::<Value>()?;
        let most_recent = Self::from_str(
            data["tag_name"]
                .as_str()
                .unwrap()
                .strip_prefix("v")
                .unwrap(),
        )
        .unwrap();
        Ok(most_recent.gt(self))
    }
}

impl Debug for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            self.version[0], self.version[1], self.version[2]
        )
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            self.version[0], self.version[1], self.version[2]
        )
    }
}

impl PartialEq<String> for Version {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        &self.to_string() == other
    }
}

impl PartialEq<Version> for String {
    #[inline]
    fn eq(&self, other: &Version) -> bool {
        self == &other.to_string()
    }
}

impl PartialEq<&str> for Version {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        &self.to_string() == other
    }
}

impl PartialEq<Version> for &str {
    #[inline]
    fn eq(&self, other: &Version) -> bool {
        self == &other.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_version_new() {
        let version = Version::new(0, 0, 0);

        assert_eq!(version, "0.0.0");
        assert_eq!(version, "0.0.0".to_string());
        assert_eq!(Some(version), Version::from_str("0.0.0"));
        assert_eq!(version.to_plain_string(), "000".to_string());
    }

    #[test]
    pub fn test_version_from_str() {
        let version = Version::from_str("0.0.0");

        assert!(version.is_some());

        let version = version.unwrap();

        assert_eq!(version, "0.0.0");
        assert_eq!(version, "0.0.0".to_string());
        assert_eq!(version, Version::new(0, 0, 0));
        assert_eq!(version.to_plain_string(), "000".to_string());
    }

    #[test]
    pub fn test_version_long() {
        let version = Version::from_str("100.0.0");

        assert!(version.is_some());

        let version = version.unwrap();

        assert_eq!(version, "100.0.0");
        assert_eq!(version, "100.0.0".to_string());
        assert_eq!(version, Version::new(100, 0, 0));
        assert_eq!(version.to_plain_string(), "10000".to_string());
    }

    #[test]
    pub fn test_incorrect_versions() {
        assert_eq!(Version::from_str(""), None);
        assert_eq!(Version::from_str(".0"), None);
        assert_eq!(Version::from_str("0."), None);
        assert_eq!(Version::from_str(".0.0"), None);
        assert_eq!(Version::from_str("0.0."), None);
    }
}
