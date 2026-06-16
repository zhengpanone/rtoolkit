use anyhow::Result;
use rtoolkit::commands::build_cli;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    build_cli()
}

#[cfg(test)]
mod tests {}
