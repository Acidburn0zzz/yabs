// Copyright (c) 2015 - 2016, Alberto Corona <ac@albertocorona.com>
// All rights reserved. This file is part of yabs, distributed under the BSD
// 3-Clause license. For full terms please see the LICENSE file.

extern crate serde;
extern crate toml;
extern crate walkdir;
extern crate ansi_term;

use desc::project::*;
use error::{YabsError, YabsErrorKind};
use ext::{Job, PrependEach, get_assumed_filename_for_dir, run_cmd, spawn_cmd};

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Child;

pub trait Buildable<T> {
    fn path(&self) -> PathBuf;
}

impl<T> Buildable<T> for Binary {
    fn path(&self) -> PathBuf {
        PathBuf::from(self.name())
    }
}

impl<T> Buildable<T> for Library {
    fn path(&self) -> PathBuf {
        self.path()
    }
}

// A build file could have multiple `Profile`s
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BuildFile {
    project: ProjectDesc,
    #[serde(rename = "bin")]
    binaries: Option<Vec<Binary>>,
    #[serde(rename = "lib")]
    libraries: Option<Vec<Library>>,
}

impl BuildFile {
    // Creates a `Profiles` from a toml file. Is essentiall `BuildFile::new`
    pub fn from_file<T: AsRef<Path>>(filepath: &T) -> Result<BuildFile, YabsError> {
        let mut buffer = String::new();
        let mut file = File::open(filepath)?;
        file.read_to_string(&mut buffer)?;
        let mut build_file: BuildFile = toml::from_str(&buffer)?;
        build_file.project.find_source_files()?;
        Ok(build_file)
    }

    pub fn print_sources(&mut self) {
        for target in self.project.file_mod_map.keys() {
            info!("{}", target.source().display());
        }
    }

    fn spawn_build_object(&self, target: &Target) -> Result<(String, Child), YabsError> {
        let command = &format!("{CC} -c {CFLAGS} {INC} -o \"{OBJ}\" \"{SRC}\"",
                CC =
                    &self.project.compiler.as_ref().unwrap_or(&String::from("gcc")),
                CFLAGS = &self.project
                              .compiler_flags
                              .as_ref()
                              .unwrap_or(&vec![])
                              .prepend_each("-")
                              .join(" "),
                INC = &self.project
                           .include
                           .as_ref()
                           .unwrap_or(&vec![])
                           .prepend_each("-I")
                           .join(" "),
                OBJ = target.object().to_str().unwrap(),
                SRC = target.source().to_str().unwrap());
        Ok((command.to_owned(), spawn_cmd(command)?))
    }

    fn build_object_queue<T: Buildable<T>>(&self,
                                           build_target: &T)
                                           -> Result<Vec<Target>, YabsError> {
        let mut queue = BTreeSet::new();
        let target_path = build_target.path();
        if target_path.exists() {
            for (target, modtime) in &self.project.file_mod_map {
                if modtime > &fs::metadata(&target_path)?.modified()? || !target.object().exists() {
                    queue.insert(target.clone());
                }
            }
        } else {
            for target in self.project.file_mod_map.keys() {
                if !target.object().exists() {
                    queue.insert(target.clone());
                }
            }
        }
        Ok(queue.iter().cloned().collect())
    }

    fn build_all_binaries(&mut self, jobs: usize) -> Result<(), YabsError> {
        if !&self.binaries.is_some() {
            return Ok(());
        }
        for binary in self.binaries.clone().unwrap() {
            let job_queue = self.build_object_queue(&binary)?;
            self.run_job_queue(job_queue, jobs)?;
            self.build_binary(&binary)?;
        }
        Ok(())
    }

    fn run_job_queue(&self, mut job_queue: Vec<Target>, jobs: usize) -> Result<(), YabsError> {
        let mut job_processes: Vec<Job> = Vec::new();
        while !job_queue.is_empty() {
            if job_processes.len() < jobs {
                if let Some(target) = job_queue.pop() {
                    let job = Job::new(self.spawn_build_object(&target)?);
                    info!("{}", job.command());
                    job_processes.push(job);
                }
            } else {
                while !job_processes.is_empty() {
                    if let Some(mut job) = job_processes.pop() {
                        job.yield_self()?;
                    }
                }
            }
        }
        while !job_processes.is_empty() {
            if let Some(mut job) = job_processes.pop() {
                job.yield_self()?;
            }
        }
        Ok(())
    }

    fn build_binary(&self, binary: &Binary) -> Result<(), YabsError> {
        let object_list = if self.binaries.as_ref().unwrap().len() == 1 {
            self.project.object_list_as_string(None)?
        } else {
            self.project
                .object_list_as_string(Some(self.binaries
                                                .clone()
                                                .unwrap()
                                                .into_iter()
                                                .filter(|bin| bin.path() != binary.path())
                                                .collect::<Vec<Binary>>()))?
        };
        Ok(run_cmd(&format!("{CC} {LFLAGS} -o {BIN} {OBJ_LIST} {LIB_DIR} {LIBS}",
                           CC = &self.project.compiler.as_ref().unwrap_or(&String::from("gcc")),
                           LFLAGS = &self.project
                                         .lflags
                                         .as_ref()
                                         .unwrap_or(&vec![])
                                         .prepend_each("-")
                                         .join(" "),
                           BIN = binary.name(),
                           OBJ_LIST = object_list,
                           LIB_DIR = &self.project
                                          .lib_dir
                                          .as_ref()
                                          .unwrap_or(&vec![])
                                          .prepend_each("-L")
                                          .join(" "),
                           LIBS = &self.project.libs_as_string()))?)
    }

    pub fn build_static_library(&self, library: &Library) -> Result<(), YabsError> {
        let object_list = &self.project.object_list_as_string(None)?;
        Ok(run_cmd(&format!("{AR} {ARFLAGS} {LIB} {OBJ_LIST}",
                           AR = &self.project.ar.as_ref().unwrap_or(&String::from("ar")),
                           ARFLAGS =
                               &self.project.arflags.as_ref().unwrap_or(&String::from("rcs")),
                           LIB = library.static_file_name().display(),
                           OBJ_LIST = object_list))?)
    }

    pub fn build_dynamic_library(&self, library: &Library) -> Result<(), YabsError> {
        let object_list = &self.project.object_list_as_string(None)?;
        Ok(run_cmd(&format!("{CC} -shared -o {LIB} {OBJ_LIST} {LIBS}",
                           CC = &self.project.compiler.as_ref().unwrap_or(&String::from("gcc")),
                           LIB = library.dynamic_file_name().display(),
                           OBJ_LIST = object_list,
                           LIBS = &self.project.libs_as_string()))?)
    }

    pub fn build_library(&self, library: &Library) -> Result<(), YabsError> {
        if library.is_static() {
            self.build_static_library(library)?;
        }
        if library.is_dynamic() {
            self.build_dynamic_library(library)?;
        }
        Ok(())
    }

    pub fn build_library_with_name(&mut self, name: &str, jobs: usize) -> Result<(), YabsError> {
        if let Some(libraries) = self.libraries.as_ref() {
            if let Some(library) = libraries.into_iter()
                                            .find(|&lib| {
                                                      lib.name() == name
                                                  }) {
                let job_queue = self.build_object_queue(library)?;
                self.run_job_queue(job_queue, jobs)?;
                self.build_library(library)?;
            }
        } else {
            bail!(YabsErrorKind::TargetNotFound("library".to_owned(), name.to_owned()))
        }
        Ok(())
    }

    pub fn build_binary_with_name(&mut self, name: &str, jobs: usize) -> Result<(), YabsError> {
        if let Some(binaries) = self.binaries.as_ref() {
            if let Some(binary) = binaries.into_iter()
                                          .find(|&bin| {
                                                    bin.name() == name
                                                }) {
                let job_queue = self.build_object_queue(binary)?;
                self.run_job_queue(job_queue, jobs)?;
                self.build_binary(binary)?;
            }
        } else {
            bail!(YabsErrorKind::TargetNotFound("binary".to_owned(), name.to_owned()))
        }
        Ok(())
    }

    pub fn build_all_libraries(&mut self, jobs: usize) -> Result<(), YabsError> {
        if !self.libraries.is_some() {
            return Ok(());
        }
        for library in self.libraries.clone().unwrap() {
            let job_queue = self.build_object_queue(&library)?;
            self.run_job_queue(job_queue, jobs)?;
            self.build_library(&library)?;
        }
        Ok(())
    }

    pub fn build(&mut self, jobs: usize) -> Result<(), YabsError> {
        self.project.run_script(&self.project.before_script)?;
        self.build_all_binaries(jobs)?;
        self.build_all_libraries(jobs)?;
        self.project.run_script(&self.project.after_script)?;
        Ok(())
    }

    pub fn clean(&self) -> Result<(), YabsError> {
        for target in self.project.file_mod_map.keys() {
            if target.object().exists() && fs::remove_file(target.object()).is_ok() {
                info!("removed object '{}'", target.object().display());
            }
        }
        if let Some(binaries) = self.binaries.clone() {
            for binary in binaries {
                let bin_path = PathBuf::from(binary.name());
                if bin_path.exists() && fs::remove_file(&bin_path).is_ok() {
                    info!("removed binary '{}'", bin_path.display());
                }
            }
        }
        if let Some(libraries) = self.libraries.clone() {
            for library in libraries {
                if library.dynamic_file_name().exists() &&
                   fs::remove_file(library.dynamic_file_name()).is_ok() {
                    info!("removed library '{}'",
                          library.dynamic_file_name().display());
                }
                if library.static_file_name().exists() &&
                   fs::remove_file(library.static_file_name()).is_ok() {
                    info!("removed library '{}'", library.static_file_name().display());
                }
            }
        }
        Ok(())
    }
}

pub fn find_build_file(dir: &mut PathBuf) -> Result<BuildFile, YabsError> {
    let original = dir.clone();
    loop {
        if let Some(filepath) = check_dir(dir) {
            env::set_current_dir(&dir)?;
            return Ok(BuildFile::from_file(&dir.join(filepath))?);
        } else if !dir.pop() {
            break;
        }
    }
    bail!(YabsErrorKind::NoAssumedToml(original.to_str().unwrap().to_owned()))
}

fn check_dir(dir: &PathBuf) -> Option<PathBuf> {
    if let Some(assumed) = get_assumed_filename_for_dir(dir) {
        if dir.join(&assumed).exists() {
            return Some(dir.join(assumed));
        }
    }
    None
}

#[test]
#[should_panic]
fn test_empty_buildfile() {
    let bf = BuildFile::from_file(&"test/empty.toml").unwrap();
    assert_eq!(bf.binaries.unwrap().len(), 0);
}

#[test]
#[should_panic]
fn test_non_empty_buildfile() {
    let bf = BuildFile::from_file(&"test/test_project/test.toml").unwrap();
    let default_proj: ProjectDesc = Default::default();
    assert_eq!(bf.project, default_proj);
}
