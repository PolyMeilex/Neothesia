mod args;
mod audio;
mod hrpt;
mod midi;
mod transkun;

use args::Backend;

fn main() -> anyhow::Result<()> {
    let args = args::Args::get_from_env()?;

    match args.backend {
        Backend::Hrpt => hrpt::run(&args),
        Backend::Transkun => transkun::run(&args),
    }
}
