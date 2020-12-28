#[macro_use]
extern crate clap;

mod app;
mod clap_app;
pub mod config;

use std::{process, io, io::Write};

use reqwest::StatusCode;

use crate::{app::App, config::delete_access_token_file};

use colmsg::dirs::PROJECT_DIRS;
use colmsg::{errors::*, Config, Group};
use colmsg::errors::ErrorKind::ReqwestError;
use colmsg::controller::Controller;
use colmsg::http::client::{SClient, SHClient, HClient};

fn run_controller<C: SHClient>(config: &Config<C>) -> Result<bool> {
    let controller = Controller::new(config);
    controller.run()
}

fn run_sakurazaka(app: &App) -> Result<bool> {
    let config: Config<SClient> = app.sakurazaka_config()?;
    match &config.group {
        Group::Sakurazaka | Group::All => run_controller(&config),
        _ => Ok(true)
    }
}

fn run_hinatazaka(app: &App) -> Result<bool> {
    let config: Config<HClient> = app.hinatazaka_config()?;
    match &config.group {
        Group::Hinatazaka | Group::All => run_controller(&config),
        _ => Ok(true)
    }
}

fn run() -> Result<bool> {
    let app = App::new()?;
    if app.matches.is_present("config-dir") {
        writeln!(io::stdout(), "{}", PROJECT_DIRS.config_dir().to_string_lossy())?;
        return Ok(true);
    }
    if app.matches.is_present("download-dir") {
        writeln!(io::stdout(), "{}", PROJECT_DIRS.download_dir().to_string_lossy())?;
        return Ok(true);
    }
    let mut result = run_sakurazaka(&app);
    loop {
        match &result {
            Err(Error(ReqwestError(re), _)) => {
                if Some(StatusCode::UNAUTHORIZED) != re.status() { break; };
                delete_access_token_file()?;
                result = run_sakurazaka(&app);
            }
            _ => { break; }
        }
    }

    if let Err(_e) = &result { return result; }

    result = run_hinatazaka(&app);
    loop {
        match &result {
            Err(Error(ReqwestError(re), _)) => {
                if Some(StatusCode::UNAUTHORIZED) != re.status() { break; };
                delete_access_token_file()?;
                result = run_hinatazaka(&app);
            }
            _ => { break; }
        }
    }

    result
}

fn main() {
    let result = run();

    match result {
        Err(error) => {
            handle_error(&error);
            process::exit(1);
        }
        Ok(false) => {
            process::exit(1);
        }
        Ok(true) => {
            process::exit(0);
        }
    }
}
