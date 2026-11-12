use std::io::{BufReader, IsTerminal, Write};
use zmachine_cli::{EXIT_USAGE, Invocation, USAGE, parse_args, play};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let config = match parse_args(&args) {
        Ok(Invocation::Run(config)) => config,
        Ok(Invocation::Help) => {
            print!("{USAGE}");
            return;
        }
        Ok(Invocation::Version) => {
            println!("zm {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        Err(message) => {
            eprint!("zm: {message}\n{USAGE}");
            std::process::exit(EXIT_USAGE);
        }
    };

    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    let mut err = std::io::stderr().lock();

    // A terminal shows what the player types; a file or pipe does not, so the
    // input is written out to keep the session readable.
    let result = match config.script.as_deref() {
        Some(path) if path.as_os_str() != "-" => match std::fs::File::open(path) {
            Ok(file) => play(&config, &mut BufReader::new(file), true, &mut out, &mut err),
            Err(e) => {
                let _ = writeln!(err, "zm: cannot read {}: {e}", path.display());
                std::process::exit(EXIT_USAGE);
            }
        },
        _ => {
            let echo = !stdin.is_terminal();
            play(&config, &mut stdin.lock(), echo, &mut out, &mut err)
        }
    };

    match result {
        Ok(status) => {
            let _ = out.flush();
            std::process::exit(status)
        }
        // A closed pipe or a full disk: nothing more can usefully be written.
        Err(_) => std::process::exit(EXIT_USAGE),
    }
}
