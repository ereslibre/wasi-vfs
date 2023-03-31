use std::path::{Path, PathBuf};

use anyhow::Result;
use structopt::StructOpt;
use tempdir::TempDir;
use uuid::Uuid;
use walkdir::WalkDir;
use std::io::Write;
use xz2::write::XzEncoder;

mod module_link;

fn parse_map_dirs(s: &str) -> anyhow::Result<(PathBuf, PathBuf)> {
    let parts: Vec<&str> = s.split("::").collect();
    if parts.len() != 2 {
        anyhow::bail!("must contain exactly one double colon ('::')");
    }
    Ok((parts[0].into(), parts[1].into()))
}

#[derive(Debug, StructOpt)]
pub enum App {
    #[structopt(setting(structopt::clap::AppSettings::Hidden))]
    LinkModule {
        #[structopt(parse(from_os_str))]
        input: PathBuf,

        #[structopt(short, parse(from_os_str))]
        output: PathBuf,
    },

    /// Package directories into Wasm module
    Pack {
        /// Whether files within the mapped directory are saved in compressed form
        #[structopt(long, short)]
        compress: bool,

        /// The input Wasm module's file path.
        #[structopt(parse(from_os_str))]
        input: PathBuf,

        /// Package a host directory into Wasm module at a guest directory
        #[structopt(long = "mapdir", value_name = "GUEST_DIR::HOST_DIR", parse(try_from_str = parse_map_dirs))]
        map_dirs: Vec<(PathBuf, PathBuf)>,

        #[structopt(skip)]
        tmp_dirs: Vec<TempDir>,

        /// The file path to write the output Wasm module to.
        #[structopt(long, short, parse(from_os_str))]
        output: PathBuf,
    },
}

impl App {
    pub fn execute(self) -> Result<()> {
        match self {
            App::LinkModule { input, .. } => {
                let bytes = std::fs::read(&input)?;
                module_link::link(&bytes);
            }
            App::Pack {
                compress,
                input,
                map_dirs,
                mut tmp_dirs,
                output,
            } => {
                std::env::set_var("__WASI_VFS_PACKING", "1");
                let mut wizer = wizer::Wizer::new();
                wizer.allow_wasi(true)?;
                wizer.init_func("wasi_vfs_pack_fs");
                wizer.inherit_stdio(true);
                wizer.inherit_env(true);
                wizer.keep_init_func(true);
                wizer.wasm_bulk_memory(true);
                for (guest_dir, host_dir) in map_dirs {
                    let host_dir = if compress {
                        let tmp_dir = TempDir::new(&Uuid::new_v4().to_string())?;
                        Self::compressed_host_dir(&host_dir, tmp_dir.path())?;
                        let res = tmp_dir.path().to_path_buf();
                        tmp_dirs.push(tmp_dir);
                        res
                    } else {
                        host_dir
                    };
                    println!("mapping {:?} to {:?}", host_dir, guest_dir);
                    wizer.map_dir(guest_dir, host_dir);
                }
                let wasm_bytes = std::fs::read(&input)?;
                let output_bytes = wizer.run(&wasm_bytes)?;
                std::fs::write(output, output_bytes)?;
            }
        }
        Ok(())
    }

    fn compressed_host_dir(
        dir: &(impl Into<PathBuf> + std::convert::AsRef<Path> + std::fmt::Debug),
        tmp_dir: &Path,
    ) -> Result<()> {
        for entry in WalkDir::new(dir) {
            let current_entry = entry?;
            let relative_path = current_entry.path().strip_prefix(&dir)?;
            let target_path = tmp_dir.join(relative_path);
            if current_entry.file_type().is_dir() {
                std::fs::create_dir_all(&target_path)?;
            } else {
                let contents = std::fs::read(current_entry.path())?;
                let mut compressed_contents = XzEncoder::new(Vec::new(), 9);
                compressed_contents.write_all(&contents)?;
                std::fs::write(&target_path, compressed_contents.finish()?)?;
            }
        }
        Ok(())
    }
}
