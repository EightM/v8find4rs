use crate::v8_app::{V8Arch, V8AppType};
use std::path::PathBuf;
use std::cmp::Ordering;
use crate::v8_finder::v8_dir::V8Dir;
use itertools::Itertools;
use std::{env, io};
use std::fs::File;
use std::io::{Read, Error, ErrorKind};
use regex::Regex;
use encoding_rs_io::DecodeReaderBytes;
use lazy_static::lazy_static;

lazy_static! {
    static ref PLATFORM_VERSION_REGEX: Regex = Regex::new(r"\d\.\d\.\d+\.\d+").unwrap();
}

#[derive(Debug)]
pub struct V8Platform {
    // 8.3 <- this .13
    pub generation: u32,
    // 8.3.13 <- this
    pub version: u32,
    // 8.3.13.1234 <- this
    pub build: u32,
    // x64 or x86
    pub arch: V8Arch,
    pub path: PathBuf,
}

impl PartialEq for V8Platform {
    fn eq(&self, other: &Self) -> bool {
        self.generation == other.generation
            && self.version == other.version
            && self.build == other.build
            && self.arch == other.arch
    }
}

impl Eq for V8Platform {}

impl PartialOrd for V8Platform {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for V8Platform {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.generation, self.version, self.build, &self.arch)
            .cmp(&(other.generation, other.version, other.build, &other.arch))
    }
}

impl V8Platform {
    pub fn get_app_by_type(&self, app_type: V8AppType) -> Option<PathBuf> {
        let current_os = env::consts::OS;
        let thick_client_path = match current_os {
            "windows" => self.path.join("bin").join(app_type.value().to_owned() + ".exe"),
            "linux" => self.path.join(app_type.value()),
            "macos" => self.path.join(app_type.value()),
            _ => PathBuf::new(),
        };

        if thick_client_path.exists() {
            Some(thick_client_path)
        } else {
            None
        }
    }

    fn from_version_path(path: PathBuf) -> Option<Self> {
        let str_path = path.to_str().unwrap_or("");
        let v8_version_group_count = 4; // 8 3 13 1234

        let v8_version = &str_path.split(r"\").last().unwrap_or("");
        if PLATFORM_VERSION_REGEX.is_match(v8_version) {
            let split_version: Vec<&str> = v8_version.split(".").collect();
            if split_version.len() == v8_version_group_count {
                let v8_platform = V8Platform {
                    generation: split_version[1].parse().unwrap(),
                    version: split_version[2].parse().unwrap(),
                    build: split_version[3].parse().unwrap(),
                    arch: V8Arch::from_path(&path),
                    path,
                };
                return Some(v8_platform);
            }
        }
        None
    }

    pub fn from_v8_dir(v8_dir: &V8Dir) -> Option<Vec<V8Platform>> {
        let mut v8_platforms = Vec::new();
        let sub_dirs: Vec<_> = v8_dir.path.read_dir().ok()?.collect();
        for dir in sub_dirs {
            if let Ok(dir) = dir {
                let dir_path = dir.path();
                let v8_platform = V8Platform::from_version_path(dir_path);
                if let Some(v8_platform) = v8_platform {
                    v8_platforms.push(v8_platform)
                }
            }
        }

        Some(v8_platforms)
    }

    pub fn v8_platforms() -> Result<Vec<V8Platform>, io::Error> {
        let v8_root_dirs = possible_v8installation_paths()?;
        let v8_root_dirs: Vec<_> = v8_root_dirs.iter()
            .filter(|v8_dir| v8_dir.path.exists())
            .collect();

        let mut all_v8_platforms = Vec::new();
        for v8_root_dir in v8_root_dirs {
            let platforms = V8Platform::from_v8_dir(v8_root_dir);
            if let Some(mut platforms) = platforms {
                all_v8_platforms.append(&mut platforms);
            }
        }
        Ok(all_v8_platforms)
    }
}

fn possible_v8installation_paths() -> Result<Vec<V8Dir>, io::Error> {
    let current_os = env::consts::OS;
    match current_os {
        "windows" => v8_windows_paths(),
        "linux" => v8_linux_paths(),
        "macos" => v8_macos_paths(),
        _ => Ok(Vec::new())
    }
}

fn v8_linux_paths() -> Result<Vec<V8Dir>, io::Error> {
    let starter_cfg_path = PathBuf::from("~/.1C/1cestart");
    let mut locations_from_starter = read_locations_from_starter(starter_cfg_path)?;
    let mut default_v8_paths = read_default_linux_paths()?;

    let mut v8_all_paths = Vec::new();
    v8_all_paths.append(&mut locations_from_starter);
    v8_all_paths.append(&mut default_v8_paths);

    Ok(v8_all_paths)
}

fn v8_macos_paths() -> Result<Vec<V8Dir>, io::Error> {
    let starter_cfg_path = PathBuf::from("~/.1C/1cestart");
    let mut locations_from_starter = read_locations_from_starter(starter_cfg_path)?;
    let mut default_v8_paths = read_default_macos_paths();

    let mut v8_all_paths = Vec::new();
    v8_all_paths.append(&mut locations_from_starter);
    v8_all_paths.append(&mut default_v8_paths);

    Ok(v8_all_paths)
}

fn v8_windows_paths() -> Result<Vec<V8Dir>, io::Error> {
    let all_users_starter = get_starter_path_windows("ALLUSERSPROFILE")?;
    let mut v8_paths_all_users = read_locations_from_starter(
        all_users_starter)?;

    let local_user_starter = get_starter_path_windows("APPDATA")?;
    let mut v8_paths_local_user = read_locations_from_starter(
        local_user_starter)?;

    let mut default_v8_paths = read_default_windows_paths()?;

    let mut v8_all_paths = Vec::new();
    v8_all_paths.append(&mut v8_paths_all_users);
    v8_all_paths.append(&mut v8_paths_local_user);
    v8_all_paths.append(&mut default_v8_paths);

    v8_all_paths = v8_all_paths.into_iter().unique().collect();

    Ok(v8_all_paths)
}

fn read_default_linux_paths() -> Result<Vec<V8Dir>, io::Error> {
    let x32_old_path = PathBuf::from("/opt/1cv8/i386");
    let x64_old_path = PathBuf::from("/opt/1cv8/x86_64");
    let x32_new_path = PathBuf::from("/opt/v8.3/i386");
    let x64_new_path = PathBuf::from("/opt/v8.3/x86_64");

    let mut v8_paths: Vec<V8Dir> = Vec::new();
    v8_paths.push(V8Dir::from_path(PathBuf::from(x32_old_path)));
    v8_paths.push(V8Dir::from_path(PathBuf::from(x64_old_path)));
    v8_paths.push(V8Dir::from_path(PathBuf::from(x32_new_path)));
    v8_paths.push(V8Dir::from_path(PathBuf::from(x64_new_path)));

    Ok(v8_paths)
}

fn read_default_macos_paths() -> Vec<V8Dir> {
    vec![V8Dir::from_path(PathBuf::from("/opt/1cv8"))]
}

fn read_default_windows_paths() -> Result<Vec<V8Dir>, io::Error> {
    let program_files_x86_var = env::var_os("PROGRAMFILES(x86)");
    let program_files_var = env::var_os("PROGRAMFILES");
    let local_appdata_var = env::var_os("LOCALAPPDATA");

    let mut v8_paths = Vec::with_capacity(7);

    if let Some(program_files_x86_path) = program_files_x86_var {
        v8_paths.push(V8Dir::from_path(
            PathBuf::from(&program_files_x86_path).join("1cv8")));

        v8_paths.push(V8Dir::from_path(
            PathBuf::from(&program_files_x86_path).join("1cv82")));
    }

    if let Some(program_files_path) = program_files_var {
        v8_paths.push(V8Dir::from_path(
            PathBuf::from(&program_files_path).join("1cv8")));
        v8_paths.push(V8Dir::from_path(
            PathBuf::from(&program_files_path).join("1cv82")));
    }

    if let Some(local_appdata_path) = local_appdata_var {
        v8_paths.push(V8Dir::from_path(
            PathBuf::from(&local_appdata_path).join("Programs").join("1cv8")));
        v8_paths.push(V8Dir::from_path(
            PathBuf::from(&local_appdata_path).join("Programs").join("1cv8_x86")));
        v8_paths.push(V8Dir::from_path(
            PathBuf::from(&local_appdata_path).join("Programs").join("1cv8_x64")));
    }

    Ok(v8_paths)
}

fn get_starter_path_windows(env_var_name: &str) -> Result<PathBuf, io::Error> {
    let starter_file_suffix = r"1C\1CEStart\1CEStart.cfg"; // Windows Vista and higher
    let config_root_dir = env::var_os(env_var_name);

    if let Some(config_dir) = config_root_dir {
        Ok(PathBuf::from(config_dir).join(starter_file_suffix))
    } else {
        Err(Error::new(ErrorKind::InvalidData, "Can't read sys variable"))
    }
}

fn read_locations_from_starter(starter_cfg_path: PathBuf) -> Result<Vec<V8Dir>, io::Error> {
    let starter_file = File::open(starter_cfg_path)?;

    let mut decoder = DecodeReaderBytes::new(starter_file);
    let mut file_content = String::new();

    let _ = decoder.read_to_string(&mut file_content);

    let installed_locations: Vec<PathBuf> = file_content.lines()
        .filter(|line| line.starts_with("InstalledLocation"))
        .map(|location| &location[location.find("=").unwrap_or(0)..])
        .map(|location| &location[1..])
        .map(|location| PathBuf::from(location))
        .collect();

    let mut v8_dirs = Vec::new();
    for location in installed_locations {
        let v8_dir = V8Dir::from_path(location);
        v8_dirs.push(v8_dir)
    };

    Ok(v8_dirs)
}
