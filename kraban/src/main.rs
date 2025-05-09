mod app;
mod cli;

use std::io::stdout;

use app::App;
use cli::cli;
use cli_log::init_cli_log;
use color_eyre::{Result, eyre::Context};
use crossterm::{event::EnableFocusChange, execute};
use kraban_config::{print_default_config, write_default_config};

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = cli();
    let is_testing = *cli.get_one("testing").expect("Option has default value");
    if *cli
        .get_one("print_default_config")
        .expect("Option has default value")
    {
        print_default_config();
        return Ok(());
    }
    if *cli
        .get_one("write_default_config")
        .expect("Option has default value")
    {
        return write_default_config(is_testing);
    };
    init_cli_log!();
    let terminal = ratatui::init();
    let result = execute!(stdout(), EnableFocusChange)
        .wrap_err("Failed to enable focus change")
        .and(App::run(terminal, cli));
    ratatui::restore();
    result
}
