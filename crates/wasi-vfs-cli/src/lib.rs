use std::path::PathBuf;

use anyhow::{anyhow, Result};
use structopt::StructOpt;
use tempdir::TempDir;
use uuid::Uuid;
use zip_archive::{Archiver, Format};

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
                        Self::compressed_host_dir(&host_dir)?
                    } else {
                        host_dir
                    };
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
        dir: &(impl Into<PathBuf> + std::convert::AsRef<std::path::Path> + std::fmt::Debug),
    ) -> Result<PathBuf> {
        let tmp_dir = TempDir::new(&Uuid::new_v4().to_string())?;
        let mut archiver = Archiver::new();
        archiver.push(dir);
        archiver.set_destination(tmp_dir.path());
        archiver.set_format(Format::Xz);
        if archiver.archive().is_ok() {
            println!("temp path is {:?}", tmp_dir);
            Ok(tmp_dir.path().to_path_buf())
        } else {
            Err(anyhow!("error archiving directory {:?}", dir))
        }
    }
}
