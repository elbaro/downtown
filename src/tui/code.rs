use color_eyre::Result;
use rustpython_ast::{text_size::TextRange, StmtAsyncFunctionDef, StmtFunctionDef, Visitor};
use rustpython_parser::Parse;

use crate::tui::LineNum;

pub struct PythonCode {
    pub path: String,
    pub source: String,
    pub funcs: Vec<(LineNum, String)>,
}

struct FunctionVisitor {
    line_offsets: Vec<usize>,
    funcs: Vec<(LineNum, String)>,
}
impl rustpython_ast::Visitor<TextRange> for FunctionVisitor {
    fn visit_stmt_function_def(&mut self, node: StmtFunctionDef<TextRange>) {
        let name = node.name.as_str().to_string();
        let byte_offset = node.range.start().to_usize();
        let line = self.line_offsets.partition_point(|&x| byte_offset > x) + 1;
        self.funcs.push((line, name));
        self.generic_visit_stmt_function_def(node)
    }
    fn visit_stmt_async_function_def(&mut self, node: StmtAsyncFunctionDef<TextRange>) {
        let name = node.name.as_str().to_string();
        let byte_offset = node.range.start().to_usize();
        let line = self.line_offsets.partition_point(|&x| byte_offset > x) + 1;
        dbg!(line, &name);
        self.funcs.push((line, name));
        self.generic_visit_stmt_async_function_def(node)
    }
}

impl PythonCode {
    pub async fn new(path: String) -> Result<Self> {
        let source = tokio::fs::read_to_string(&path).await?;
        let mut offset = 0;
        let mut line_offsets = vec![];
        for line in source.split_inclusive('\n') {
            offset += line.len();
            line_offsets.push(offset);
        }

        let suite = rustpython_ast::Suite::parse(&source, &path)?;

        let mut visitor = FunctionVisitor {
            line_offsets,
            funcs: Default::default(),
        };

        for stmt in suite.into_iter() {
            visitor.visit_stmt(stmt);
        }

        Ok(Self {
            path,
            source,
            funcs: visitor.funcs,
        })
    }

    pub fn jump_to_prev_fn(&self, line: usize) -> Option<usize> {
        let idx = match self.funcs.binary_search_by_key(&line, |&(line, _)| line) {
            Ok(idx) => idx,
            Err(idx) => idx,
        };

        if idx == 0 {
            None
        } else {
            Some(self.funcs[idx - 1].0)
        }
    }

    pub fn jump_to_next_fn(&self, line: usize) -> Option<usize> {
        match self.funcs.binary_search_by_key(&line, |&(line, _)| line) {
            Ok(idx) => {
                if idx + 1 == self.funcs.len() {
                    None
                } else {
                    Some(self.funcs[idx + 1].0)
                }
            }
            Err(idx) => {
                if idx < self.funcs.len() {
                    Some(self.funcs[idx].0)
                } else {
                    None
                }
            }
        }
    }
}
