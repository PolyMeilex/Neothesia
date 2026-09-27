use std::path::PathBuf;

fn print_help() {
    let help = [
        "  -i, --input <audio-input-file>",
        "  -o, --output <midi-output-file>",
        "  -m, --model <model-file>",
    ];
    println!("Options:");
    println!("{}", help.join("\n"));
    println!();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// High-resolution Piano Transcription (Q. Kong, et al.)
    /// https://arxiv.org/pdf/2010.01815
    Hrpt,
    /// Transkun V2 (Y. Yan, Z. Duan)
    /// https://arxiv.org/pdf/2404.09466
    /// https://openreview.net/pdf?id=DGA8XbJ8FVd
    Transkun,
}

#[derive(Debug)]
pub struct Args {
    pub input: PathBuf,
    pub output: PathBuf,
    pub model: PathBuf,
    pub backend: Backend,
}

impl Args {
    pub fn get_from_env() -> anyhow::Result<Args> {
        let mut args = std::env::args().skip(1);

        let mut input = None;
        let mut output = None;
        let mut model = None;
        let mut backend = Backend::Hrpt;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--input" | "-i" => {
                    input = args.next();
                }
                "--output" | "-o" => {
                    output = args.next();
                }
                "--model" | "-m" => {
                    model = args.next();
                    if let Some(model) = model.as_ref() {
                        backend = match model {
                            m if m.ends_with(".rten") => Backend::Hrpt,
                            m if m.ends_with(".onnx") => Backend::Transkun,
                            other => anyhow::bail!("unknown backend for model {other:?}"),
                        }
                    }
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
            backend,
        })
    }
}
