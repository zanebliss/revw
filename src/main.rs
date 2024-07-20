use crate::display::Fd;
use clap::Parser;
use display::{Color, Logger};
use git::Client;
use runners::{Ruby, Runner};
use std::{
    io::Error,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

mod core;
mod display;
mod git;
mod runners;

fn main() -> Result<(), Error> {
    let term = Arc::new(AtomicBool::new(false));
    let cli = Cli::parse();
    let client = Client::new(
        cli.commit
            .unwrap_or(git::DEFAULT_COMMIT.to_string())
            .to_string(),
    );
    let runner = Runner {
        runnable: Ruby::Rspec,
    };
    let mut logger = Logger::new();

    signal_hook::flag::register(signal_hook::consts::SIGINT, term.clone())?;

    while !term.load(Ordering::SeqCst) {
        match core::do_work(&client, &mut logger, &runner, term) {
            Err(err) => {
                logger
                    .with_stream(Fd::Stderr)
                    .with_color(Color::Red)
                    .with_text(format!("Failed with error: {err}\n"))
                    .call();
                logger
                    .with_color(Color::Red)
                    .with_text("Exiting...".to_string())
                    .call();
            }
            _ => {}
        }

        // break after first iteration because work is not performed in a continuous loop
        break;
    }

    core::cleanup(&client)?;

    Ok(())
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    commit: Option<String>,

    #[arg(short, long)]
    runner: Option<String>,
}
