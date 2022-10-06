use anyhow::Result;
use chrono::prelude::*;
use diffy::{apply, create_patch, merge, Patch};
use fs_extra::file::{read_to_string, write_all};
use serde_derive::{Deserialize, Serialize};
use std::env::var_os;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use toml::*;

#[macro_use]
extern crate log;

#[derive(Deserialize, Serialize)]
struct Config {
    win_terminal: Option<ConfigPair>,
    nu_config: Option<ConfigPair>,
    nu_env: Option<ConfigPair>,
    helix_config: Option<ConfigPair>,
    helix_languages: Option<ConfigPair>,
    ssh_config: Option<ConfigPair>,
}

#[derive(Deserialize, Serialize)]
struct ConfigPair {
    source: String,
    destination: String,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "config_sync")]
struct Opt {
    #[structopt(short = "c", parse(from_os_str))]
    conf: Option<PathBuf>,
}

fn main() -> Result<()> {
    env_logger::init();

    info!("getting options from structopt");
    let opt = Opt::from_args();

    info!("running get_toml_path to check the path to the toml config file on the local system");
    let toml_config_path = get_toml_path(opt.conf)?;

    let toml_config = get_toml(toml_config_path)?;

    for conf in toml_config {
        build_and_apply_patch(&conf.source, &conf.destination)?;
    }

    Ok(())
}

fn get_toml_path(user_path: Option<PathBuf>) -> Result<PathBuf> {
    let os_conf_path = if cfg!(windows) {
        var_os("HOMEPATH")
    } else {
        var_os("HOME")
    };

    let mut toml_path = match user_path {
        Some(val) => val,
        None => PathBuf::from(os_conf_path.unwrap()),
    };

    toml_path.push(".config_sync.toml");

    Ok(toml_path)
}

fn get_toml(conf_path: PathBuf) -> Result<Vec<ConfigPair>> {
    let toml_conf: Config;

    // see if toml file exists
    if Path::new(&conf_path).is_file() {
        // if so, get the contents
        let toml_contents_string = read_to_string(conf_path)?;

        toml_conf = toml::from_str(&toml_contents_string)?;
    } else {
        // if not, create a default file
        toml_conf = Config {
            win_terminal: None,
            nu_config: None,
            nu_env: None,
            helix_config: None,
            helix_languages: None,
            ssh_config: None,
        };

        // blank config
        let blank_config = r#"

#[Windows path example]
#source = "C:/Users/Tom/OneDrive/configs/config"
#destination = "C:/Users/Tom/AppData/Roaming/config"

#[Unix path example]
#source = "/users/Tom/OneDrive/configs/config" #MacOS
#destination = "/home/Tom/.local/configs/config" #Linux

#[win_terminal]
#source = ""
#destination = ""

#[nu_config]
#source = ""
#destination = ""

#[nu_env]
#source = ""
#destination = ""

#[helix_config]
#source = ""
#destination = ""

#[helix_languages]
#source = ""
#destination = ""

#[ssh_config]
#source = ""
#destination = ""

"#;

        //write it to disc
        write_all(conf_path, &blank_config)?;
    }

    // convert Config to Vec<ConfigPair>
    let mut vec_conf: Vec<ConfigPair> = Vec::new();

    match toml_conf.win_terminal {
        Some(x) => vec_conf.push(x),
        None => {}
    };

    match toml_conf.nu_config {
        Some(x) => vec_conf.push(x),
        None => {}
    };

    match toml_conf.nu_env {
        Some(x) => vec_conf.push(x),
        None => {}
    };

    match toml_conf.helix_config {
        Some(x) => vec_conf.push(x),
        None => {}
    };

    match toml_conf.helix_languages {
        Some(x) => vec_conf.push(x),
        None => {}
    };

    match toml_conf.ssh_config {
        Some(x) => vec_conf.push(x),
        None => {}
    };

    // return the toml values
    Ok(vec_conf)
}

fn build_and_apply_patch(source: &str, dest: &str) -> Result<()> {
    // read in contents of source and dest to string
    let source_contents = read_to_string(source)?;

    let dest_contents = read_to_string(dest)?;

    // build patch
    let patch = create_patch(&dest_contents, &source_contents);

    // apply patch
    let updated_contents = apply(&dest_contents, &patch)?;

    // save modified dest to file
    write_all(&dest, &updated_contents)?;

    Ok(())
}
