pub mod hist;
pub mod types;
use types::*;

use std::collections::{HashMap, HashSet};

use color_eyre::Result;
use libbpf_rs::Link;
use tonari_actor::{Actor, Addr, Context};

use crate::{
    profiler::{
        hist::{Histogram, Summary},
        python::PythonSkel,
    },
    tui::{LineNum, Tui, TuiMessage},
};

pub mod python {
    include!(concat!(env!("OUT_DIR"), "/python.skel.rs"));
}

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
pub enum ProfilerMessage {
    Attach(ProfileTarget),
    Detach(ProfileTarget),
    ToggleFilter(Filter),
    // AddFilter(Filter),
    // DelFilter(Filter),
    Observe,
}

pub struct Profiler {
    skel: PythonSkel<'static>,
    links: HashMap<Probe, Link>,
    tui_addr: Addr<Tui>,
}

impl Profiler {
    pub fn new(python_code: String, tui_addr: Addr<Tui>) -> Result<Self> {
        let skel_builder = python::PythonSkelBuilder::default();
        let mut open_skel: python::OpenPythonSkel = skel_builder.open()?;

        // 127 bytes [0..=126] + 1 byte [127] = 128 bytes
        let len = python_code.len().min(127);
        open_skel.bss().FILTER_FILENAME[..len]
            .copy_from_slice(unsafe { std::mem::transmute(&python_code.as_bytes()[..len]) });
        open_skel.bss().FILTER_FILENAME[len] = 0; // NUL-terminated
        let skel = open_skel.load()?;

        Ok(Self {
            skel,
            links: Default::default(),
            tui_addr,
        })
    }
}
impl Actor for Profiler {
    type Message = ProfilerMessage;
    type Error = color_eyre::Report;
    type Context = Context<Self::Message>;

    fn handle(
        &mut self,
        _context: &mut Self::Context,
        message: Self::Message,
    ) -> std::result::Result<(), Self::Error> {
        match message {
            ProfilerMessage::Attach(target) => {
                // Attach FunctionReturn first
                for usdt in UsdtName::ALL.iter().rev() {
                    self.links.insert(
                        Probe {
                            target: target.clone(),
                            usdt_name: *usdt,
                        },
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
                    );
                }
            }
            ProfilerMessage::Detach(target) => {
                for usdt in UsdtName::ALL.iter().rev() {
                    self.links.remove(&Probe {
                        target: target.clone(),
                        usdt_name: *usdt,
                    });
                }
            }
            ProfilerMessage::ToggleFilter(filter) => {
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
            ProfilerMessage::Observe => {
                let maps = self.skel.maps();
                let allowed_lines: HashSet<u64> = maps
                    .filter_map()
                    .keys()
                    .map(|bytes| u64::from_ne_bytes(bytes.try_into().unwrap()))
                    .collect();
                let mut histograms = HashMap::<Filter, Histogram>::new();
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
                        let filter = Filter {
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
                self.tui_addr.send(TuiMessage::Profile(summary_map))?;
            }
        }
        Ok(())
    }

    fn name() -> &'static str {
        "Profiler"
    }
}
