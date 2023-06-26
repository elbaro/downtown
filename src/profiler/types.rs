use std::collections::HashMap;

use libbpf_rs::Link;

use crate::{profiler::hist::Summary, tui::LineNum};

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
pub struct Filter {
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
