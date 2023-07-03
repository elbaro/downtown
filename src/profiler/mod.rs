pub mod hist;
pub mod types;
use crate::{
    profiler::{
        hist::{Histogram, Summary},
        python::PythonSkel,
    },
    tui::LineNum,
};
use types::*;

pub mod python {
    include!(concat!(env!("OUT_DIR"), "/python.skel.rs"));
}

use color_eyre::Result;
use std::collections::{HashMap, HashSet};
use xtra::prelude::*;

#[derive(Debug)]
#[allow(non_camel_case_types)]
#[repr(C)]
struct HistogramKey {
    // pub filename: [i8; 128],
    // pub funcname: [i8; 64],
    pub lineno: u64,
    pub bucket: u64,
}

#[derive(Debug)]
pub enum ProfileAttribute {
    Latency,
    Gc,
    Malloc,
}

// messages with return=Result<()>
#[derive(Debug)]
pub enum ProfilerCommand {
    Attach(ProfileTarget),
    Detach(ProfileTarget),
    ToggleFilter(Target),
    ToggleAttribute(ProfileAttribute),
    // AddFilter(Filter),
    // DelFilter(Filter),
}

pub mod request {
    pub struct Observe;
    pub struct ListFilters {
        _file: String,
    }
}

#[derive(Actor)]

pub struct Profiler {
    skel: PythonSkel<'static>,
    links: HashMap<Probe, SendableLink>,
}

pub fn hash_path(s: &str) -> u64 {
    let mut h = 0u64;
    for b in s.bytes() {
        h = h.wrapping_mul(37).wrapping_add(b as u64);
    }
    h
}

impl Profiler {
    pub fn new(python_code: String) -> Result<Self> {
        let skel_builder = python::PythonSkelBuilder::default();
        let mut open_skel: python::OpenPythonSkel = skel_builder.open()?;
        open_skel.bss().FILTER_PATH_HASH = hash_path(&python_code);
        let skel = open_skel.load()?;

        Ok(Self {
            skel,
            links: Default::default(),
        })
    }
}

#[async_trait::async_trait]
impl Handler<ProfilerCommand> for Profiler {
    type Return = Result<()>;
    async fn handle(&mut self, cmd: ProfilerCommand, _: &mut Context<Self>) -> Self::Return {
        match cmd {
            ProfilerCommand::Attach(target) => {
                // Attach FunctionReturn first
                for usdt in UsdtName::ALL.iter().rev() {
                    self.links.insert(
                        Probe {
                            target: target.clone(),
                            usdt_name: *usdt,
                        },
                        SendableLink::new(
                            self.skel
                                .obj
                                .prog_mut(usdt.as_str())
                                .expect("bpf program error")
                                .attach_usdt(
                                    target.pid,
                                    &target.python_bin,
                                    USDT_PROVIDER,
                                    usdt.as_str(),
                                )?,
                        ),
                    );
                }
            }
            ProfilerCommand::Detach(target) => {
                for usdt in UsdtName::ALL.iter().rev() {
                    self.links.remove(&Probe {
                        target: target.clone(),
                        usdt_name: *usdt,
                    });
                }
            }
            ProfilerCommand::ToggleFilter(filter) => {
                let mut maps = self.skel.maps_mut();
                let lineno = filter.lineno.to_ne_bytes();

                if maps
                    .filter_map()
                    .lookup(&lineno, libbpf_rs::MapFlags::ANY)?
                    .is_none()
                {
                    maps.filter_map()
                        .update(&lineno, &lineno, libbpf_rs::MapFlags::ANY)?;
                } else {
                    maps.filter_map().delete(&lineno)?;
                }
            }
            ProfilerCommand::ToggleAttribute(_) => {
                todo!()
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<request::Observe> for Profiler {
    type Return = Result<SummaryMap>;
    async fn handle(&mut self, _: request::Observe, _: &mut Context<Self>) -> Self::Return {
        let maps = self.skel.maps();
        let allowed_lines: HashSet<u64> = maps
            .filter_map()
            .keys()
            .map(|bytes| u64::from_ne_bytes(bytes.try_into().unwrap()))
            .collect();
        let mut histograms = HashMap::<Target, Histogram>::new();
        let mut summary_map = HashMap::<LineNum, Summary>::new();
        let map = maps.latency_map();
        let mut keys_to_delete = vec![];
        for bytes_key in map.keys() {
            let value = map.lookup(&bytes_key, libbpf_rs::MapFlags::ANY)?.unwrap();
            let value = u64::from_ne_bytes(value.try_into().unwrap());

            unsafe {
                let key: *const HistogramKey = std::mem::transmute(bytes_key.as_ptr());
                // let filename = std::ffi::CStr::from_ptr((*key).filename.as_ptr())
                //     .to_str()
                //     .unwrap();
                // let funcname = std::ffi::CStr::from_ptr((*key).funcname.as_ptr())
                //     .to_str()
                //     .unwrap();
                let lineno = (*key).lineno;

                if !allowed_lines.contains(&lineno) {
                    keys_to_delete.push(bytes_key);
                    continue;
                }

                let bucket = (*key).bucket;
                let filter = Target {
                    // filename: filename.to_string(),
                    // funcname: funcname.to_string(),
                    lineno: lineno as usize,
                };
                histograms
                    .entry(filter)
                    .or_default()
                    .add(bucket as usize, value);
            }
        }
        let mut maps = self.skel.maps_mut();
        for key in keys_to_delete {
            maps.latency_map().delete(&key)?;
        }

        for (filter, hist) in histograms {
            summary_map.insert(filter.lineno, hist.summary());
        }
        Ok(summary_map)
    }
}
