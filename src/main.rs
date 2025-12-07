use crate::cli::schema::Cli;
use crate::config::load::{ConfigInterface, ImplConfigInterface};
use crate::ui::render::App;
use crate::ui::render::run_app;
use clap::Parser;
use color_eyre::config::HookBuilder;
use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use custom_logger as log;
use ratatui::Terminal;
use ratatui::prelude::{Backend, CrosstermBackend};
use std::io::stdout;

mod cli;
mod config;
mod error;
mod handlers;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let config = args.config;
    let impl_config = ImplConfigInterface {};

    // setup logging
    log::Logging::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .expect("log should initialize");

    // read and parse config
    let params = impl_config.read(config);
    if params.is_err() {
        log::error!("{}", params.err().unwrap());
        std::process::exit(1);
    }

    let level = match params.as_ref().unwrap().log_level.as_str() {
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        &_ => log::LevelFilter::Info,
    };

    // override level if other than info
    if level == log::LevelFilter::Debug || level == log::LevelFilter::Trace {
        let _ = log::Logging::new().with_level(level).init();
    }

    log::info!("application : {}", env!("CARGO_PKG_NAME"));
    log::info!("author      : {}", env!("CARGO_PKG_AUTHORS"));
    log::info!("version     : {}", env!("CARGO_PKG_VERSION"));
    log::info!("log-level   : {}", level.to_string().to_lowercase());
    println!();

    // start tui
    init_error_hooks()?;
    let mut terminal = init_terminal()?;
    let mut app = App::new("node metrics".to_owned(), params.unwrap());
    let res = run_app(&mut terminal, &mut app).await;
    restore_terminal()?;
    if let Err(err) = res {
        log::error!("{err:?}");
    }
    Ok(())
}

fn init_error_hooks() -> color_eyre::Result<()> {
    let (panic, error) = HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = restore_terminal();
        error(e)
    }))?;
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic(info);
    }));
    Ok(())
}

fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> color_eyre::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
