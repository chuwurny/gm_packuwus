use std::{
    ffi::{c_char, c_int, c_void, CString},
    fmt::Debug,
    fs::File,
    io::Read,
    mem::transmute,
    path::{Component, PathBuf},
    ptr::null_mut,
};

use goblin::elf::Elf;
use procfs::{
    process::{MemoryMap, Process},
    ProcError,
};

#[derive(thiserror::Error, Debug)]
pub enum ModuleFromMemMapError {
    #[error("Memory map not found")]
    NotFound,
}

#[derive(thiserror::Error, Debug)]
pub enum ModuleFromProcessError {
    #[error("Failed to get process maps: {0}")]
    Maps(ProcError),
    #[error("Failed to create module instance from memory map: {0}")]
    MemMap(ModuleFromMemMapError),
    #[error("Module in process not found")]
    NotFound,
}

#[derive(thiserror::Error, Debug)]
pub enum SymbolError {
    #[error("Failed to open file: {0}")]
    OpenFile(std::io::Error),
    #[error("Failed to read file: {0}")]
    ReadFile(std::io::Error),
    #[error("Failed to parse file's ELF header: {0}")]
    ParseFile(goblin::error::Error),
    #[error("Symbol not found")]
    NotFound,
}

#[derive(thiserror::Error, Debug)]
pub enum InterfaceError {
    #[error("Failed to find \"CreateInterface\" symbol: {0}")]
    FailedToFindSymbol(SymbolError),
    #[error("Unexpected \\0 char in interface name")]
    UnexpectedNulInName,
}

pub(crate) struct Module {
    pub path: PathBuf,
    pub start_address: u64,
    pub size: u64,
}

#[allow(dead_code)]
impl Module {
    pub fn from_mem_map(mem_map: &MemoryMap) -> Result<Module, ModuleFromMemMapError> {
        match &mem_map.pathname {
            procfs::process::MMapPath::Path(path) => Ok(Module {
                path: path.as_path().to_owned(),
                start_address: mem_map.address.0,
                size: mem_map.address.1 - mem_map.address.0,
            }),
            _ => Err(ModuleFromMemMapError::NotFound),
        }
    }

    pub fn from_process(
        process: &Process,
        filename: &str,
    ) -> Result<Module, ModuleFromProcessError> {
        if let Some(map) = process
            .maps()
            .or_else(|err| Err(ModuleFromProcessError::Maps(err)))?
            .into_iter()
            .find(|map| match &map.pathname {
                procfs::process::MMapPath::Path(map_path) => {
                    match map_path.as_path().components().last() {
                        Some(map_last_component) => match map_last_component {
                            Component::Normal(map_filename) => {
                                return map_filename.to_str().unwrap_or("") == filename
                            }
                            _ => false,
                        },
                        _ => false,
                    }
                }
                _ => false,
            })
        {
            Module::from_mem_map(&map).or_else(|err| Err(ModuleFromProcessError::MemMap(err)))
        } else {
            Err(ModuleFromProcessError::NotFound)
        }
    }

    unsafe fn memory_slice(&self) -> &[u8] {
        std::slice::from_raw_parts(self.start_address as *const u8, self.size as usize)
    }

    pub fn symbol(&self, name: &str) -> Result<*const c_void, SymbolError> {
        let mut file = File::open(&self.path).or_else(|err| Err(SymbolError::OpenFile(err)))?;

        let mut buf = vec![];

        file.read_to_end(&mut buf)
            .or_else(|err| Err(SymbolError::ReadFile(err)))?;

        let elf = Elf::parse(&buf).or_else(|err| Err(SymbolError::ParseFile(err)))?;

        Ok(elf
            .syms
            .iter()
            .find_map(|sym| {
                if let Some(sym_name) = elf.strtab.get_at(sym.st_name) {
                    if sym_name == name {
                        return Some(self.start_address + sym.st_value);
                    }
                }

                None
            })
            .ok_or_else(|| SymbolError::NotFound)? as *const c_void)
    }

    pub fn interface<T>(&self, name: &str) -> Result<*const T, InterfaceError> {
        let name = CString::new(name).or_else(|_| Err(InterfaceError::UnexpectedNulInName))?;

        Ok(unsafe {
            transmute::<_, unsafe extern "C" fn(*const c_char, *mut c_int) -> *const c_void>(
                self.symbol("CreateInterface")
                    .or_else(|err| Err(InterfaceError::FailedToFindSymbol(err)))?,
            )(name.as_ptr(), null_mut()) as *const T
        })
    }
}
