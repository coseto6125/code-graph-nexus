use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    Branch,
    Tag,
    Pr,
    Commit,
}

impl SourceType {
    fn as_str(self) -> &'static str {
        match self {
            SourceType::Branch => "branch",
            SourceType::Tag => "tag",
            SourceType::Pr => "pr",
            SourceType::Commit => "commit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitDirName {
    pub source_type: SourceType,
    pub source_id: Option<String>,
    pub sha: [u8; 20],
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("dir name missing __ separator")]
    NoSha,
    #[error("sha segment not 40-hex")]
    InvalidSha,
    #[error("prefix missing source_type")]
    NoTypeId,
    #[error("unknown source_type: {0}")]
    UnknownSourceType(String),
}

impl CommitDirName {
    pub fn parse(name: &str) -> Result<Self, ParseError> {
        let (prefix, sha_str) = name.rsplit_once("__").ok_or(ParseError::NoSha)?;
        if sha_str.len() != 40 || !sha_str.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ParseError::InvalidSha);
        }
        let mut sha = [0u8; 20];
        for i in 0..20 {
            sha[i] = u8::from_str_radix(&sha_str[i * 2..i * 2 + 2], 16)
                .map_err(|_| ParseError::InvalidSha)?;
        }

        if prefix == "commit" {
            return Ok(Self {
                source_type: SourceType::Commit,
                source_id: None,
                sha,
            });
        }

        let (type_str, id_str) = prefix.split_once('_').ok_or(ParseError::NoTypeId)?;
        let source_type = match type_str {
            "branch" => SourceType::Branch,
            "tag" => SourceType::Tag,
            "pr" => SourceType::Pr,
            other => return Err(ParseError::UnknownSourceType(other.into())),
        };
        Ok(Self {
            source_type,
            source_id: Some(id_str.into()),
            sha,
        })
    }

    pub fn format(&self) -> String {
        let sha_hex = self.sha_hex();
        match (&self.source_type, &self.source_id) {
            (SourceType::Commit, _) => format!("commit__{sha_hex}"),
            (t, Some(id)) => format!("{}_{id}__{sha_hex}", t.as_str()),
            (t, None) => format!("{}__{sha_hex}", t.as_str()),
        }
    }

    pub fn sha_hex(&self) -> String {
        self.sha.iter().map(|b| format!("{b:02x}")).collect()
    }
}
