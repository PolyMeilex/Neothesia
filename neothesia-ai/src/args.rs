use std::path::PathBuf;

fn print_help() {
    let help = [
        "  -i, --input <audio-input-file>",
        "  -o, --output <midi-output-file>",
        "  -m, --model <rten-model-file>",
    ];
    println!("Options:");
    println!("{}", help.join("\n"));
    println!();
}

#[derive(Debug)]
pub struct Args {
    pub input: PathBuf,
    pub output: PathBuf,
    pub model: PathBuf,
}

impl Args {
    pub fn get_from_env() -> anyhow::Result<Args> {
        let mut args = std::env::args().skip(1);

        let mut input = None;
        let mut output = None;
        let mut model = None;

        loop {
            let Some(arg) = args.next() else {
                break;
            };

            match arg.as_str() {
                "--input" | "-i" => {
                    input = args.next();
                }
                "--output" | "-o" => {
                    output = args.next();
                }
                "--model" | "-m" => {
                    model = args.next();
                }
                "--help" | "-h" => {
                    print_help();
                }
                _ => {}
            }
        }

        let Some(input) = input else {
            anyhow::bail!("`--input` audio file missing");
        };

        let Some(output) = output else {
            anyhow::bail!("`--output` midi file missing");
        };

        let Some(model) = model else {
            anyhow::bail!("`--model` rten model file missing");
        };

        Ok(Args {
            input: PathBuf::from(input),
            output: PathBuf::from(output),
            model: PathBuf::from(model),
        })
    }
}
