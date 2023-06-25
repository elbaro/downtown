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
