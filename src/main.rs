pub mod installer;

use colored::Colorize;
use installer::{InstallError, Installer};
use url::Url;

macro_rules! outputln {
    ($format:literal $(, $arg:tt)*) => {
        eprintln!(concat!("[{}] ", $format), "installer".bold().cyan(), $($arg)*);
    };
    ($col:ident, $format:literal $(, $arg:tt)*) => {
        eprintln!(concat!("[{}] ", $format), "installer".bold().$col(), $($arg)*);
    };
}

macro_rules! output {
    ($format:literal $(, $arg:tt)*) => {
        eprint!(concat!("[{}] ", $format), "installer".bold().cyan(), $($arg)*);
    };
    ($col:ident, $format:literal $(, $arg:tt)*) => {
        eprint!(concat!("[{}] ", $format), "installer".bold().$col(), $($arg)*);
    };
}

pub(crate) use output;
pub(crate) use outputln;

fn usage(program_name: &str, message: Option<String>) -> ! {
    outputln!("usage: {} <github-link>", program_name);
    outputln!("  github-link: The link to a C++ project that uses CMake.");
    outputln!("               This project will be git cloned and installed system-wide.");
    if let Some(msg) = message {
        outputln!("reason: {}", msg);
    }
    std::process::exit(-1);
}

fn main() {
    let mut argv = std::env::args();
    let program_name = argv.next().unwrap_or("cppinstall".into());

    //  NOTE: We check for 2 because the first argument is always
    //  going to be the program name.
    if argv.len() < 1 {
        usage(&program_name, Some("Too little arguments".into()));
    }

    let link = match argv.next() {
        Some(link) => link,
        None => {
            usage(&program_name, Some("could not find url argument.".into()));
        }
    };

    let url = match Url::parse(&link) {
        Ok(url) => url,
        Err(e) => {
            usage(
                &program_name,
                Some(format!("invalid url: {} ({})", e, link)),
            );
        }
    };

    if url.host_str() != Some("github.com") {
        usage(&program_name, Some("host must be github.com".into()));
    }

    let installer = match Installer::new(&url) {
        Ok(installer) => installer,
        Err(e) => {
            outputln!("failed to install project.");
            let e = e.to_string();
            outputln!("{}", e);
            return;
        }
    };

    outputln!(green, "successfully installed project at {}", link);
    let tmp_path = installer.temp_path();
    outputln!(
        green,
        "the temporary folder used to install it is at {}",
        tmp_path
    );
    outputln!(
        green,
        "note: use `sudo rm -rf /tmp/cppinstall-*` to remove any temporary directories."
    );
}
