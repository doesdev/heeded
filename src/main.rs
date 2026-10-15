use std::process::ExitCode;

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();

    if argv.first().map(String::as_str) == Some("hook") {
        return match heeded::hook() {
            Ok(emitted) => {
                if let Some(out) = emitted.stdout {
                    println!("{out}");
                }
                if let Some(err) = emitted.stderr {
                    eprintln!("{err}");
                }
                ExitCode::from(emitted.code as u8)
            }
            Err(ex) => {
                eprintln!("{ex}");
                ExitCode::FAILURE
            }
        };
    }

    match heeded::cli::run(&argv) {
        Ok(()) => ExitCode::SUCCESS,
        Err(ex) => {
            eprintln!("{ex}");
            ExitCode::FAILURE
        }
    }
}
