use std::path::Path;
use std::process;

mod cli;
mod config;
mod diagnostic;
mod discovery;
mod layout;
mod locale;
mod report;
mod validate;

fn main() {
    let args = cli::parse();
    match run(&args) {
        Ok(has_errors) => {
            if has_errors {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(2);
        }
    }
}

fn run(args: &cli::Args) -> Result<bool, Box<dyn std::error::Error>> {
    let config = config::resolve(args)?;
    let path = Path::new(&args.path);
    let detected_layout = layout::detect(path, config.layout)?;
    let ctx = discovery::discover(path, &detected_layout, &config)?;
    let diagnostics = validate::run_all(&ctx);
    let has_errors = report::render(&diagnostics, &ctx, &config, args)?;
    Ok(has_errors)
}
