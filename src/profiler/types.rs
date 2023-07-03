use std::collections::HashMap;

use libbpf_rs::Link;

use crate::{
    profiler::{hash_path, hist::Summary},
    tui::LineNum,
};

pub const USDT_PROVIDER: &str = "python";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProfileTarget {
    pub pid: i32,
    pub python_bin: String,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum UsdtName {
    FunctionEntry,
    FunctionReturn,
}

impl UsdtName {
    pub const ALL: &[UsdtName] = &[UsdtName::FunctionEntry, UsdtName::FunctionReturn];

    pub fn as_str(&self) -> &'static str {
        match self {
            UsdtName::FunctionEntry => "function__entry",
            UsdtName::FunctionReturn => "function__return",
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct Probe {
    pub target: ProfileTarget,
    pub usdt_name: UsdtName,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Target {
    // pub filename: String,
    // pub funcname: String,
    pub lineno: usize,
}

// Since we only hold Link for reference-count purposes, it's Sendable.
pub struct SendableLink(Link);
impl SendableLink {
    pub fn new(link: Link) -> Self {
        Self(link)
    }
}
unsafe impl Send for SendableLink {}

pub type SummaryMap = HashMap<LineNum, Summary>;

// TODO: convert to enum
pub struct Precondition {
    pub file_path: Option<String>,
    pub path_hash: Option<u64>,
    pub func_name: Option<String>,
    pub line: Option<u64>,
    pub sample_every_n: u64,
    _private: (),
}

// 1. path:func:line
// 2. func
impl Precondition {
    pub fn by_path_func_line(
        file_path: String,
        func_name: String,
        line: u64,
        sample_every_n: u64,
    ) -> Self {
        let path_hash = hash_path(&file_path);
        Self {
            file_path: Some(file_path),
            func_name: Some(func_name),
            line: Some(line),
            path_hash: Some(path_hash),
            sample_every_n,
            _private: (),
        }
    }
    pub fn by_func_name(func_name: String, sample_every_n: u64) -> Self {
        Self {
            file_path: None,
            func_name: Some(func_name),
            line: None,
            path_hash: None,
            sample_every_n,
            _private: (),
        }
    }
}
