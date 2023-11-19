pub mod installer;
pub mod registry;

use colored::Colorize;
use installer::Installer;
use registry::*;
use url::Url;

macro_rules! outputln {
    ($format:literal $(, $arg:tt)*) => {
        eprintln!(concat!("[{}] ", $format), "installer".bold().cyan() $(, $arg)*)
    };
    ($col:ident, $format:literal $(, $arg:tt)*) => {
        eprintln!(concat!("[{}] ", $format), "installer".bold().$col() $(, $arg)*)
    };
}

macro_rules! output {
    ($format:literal $(, $arg:tt)*) => {
        eprint!(concat!("[{}] ", $format), "installer".bold().cyan() $(, $arg)*)
    };
    ($col:ident, $format:literal $(, $arg:tt)*) => {
        eprint!(concat!("[{}] ", $format), "installer".bold().$col() $(, $arg)*)
    };
}

pub(crate) use output;
pub(crate) use outputln;

fn usage(program_name: &str, message: Option<String>) -> ! {
    outputln!("usage: {} [...options]", program_name);
    outputln!("  [url]: A github URL to a project that is using CMake or Make.");
    outputln!("  [package]: The name of a package name learnt from `--list-packages`");
    outputln!("  [--list-packages [...opts]]: Skip installation and output all known packages.");
    outputln!("    [filter]: The filter to apply when listing packages. This just checks if the package name contains that string.");
    if let Some(msg) = message {
        outputln!("reason: {}", msg);
    }
    std::process::exit(-1);
}

fn main() {
    let registry = PackageRegistry::default();
    let mut argv = std::env::args();
    let program_name = argv.next().unwrap_or("cinstall".into());

    //  NOTE: We check for 2 because the first argument is always
    //  going to be the program name.
    if argv.len() < 1 {
        usage(&program_name, Some("Too little arguments".into()));
    }

    let first_arg = match argv.next() {
        Some(data) => data,
        None => usage(
            &program_name,
            Some("could not find package name/url argument.".into()),
        ),
    };

    if first_arg == "--list-packages" {
        let mut filter: Option<String> = None;
        if let Some(next) = argv.next() {
            // expect this to be a filter.
            filter = Some(next);
        }
        for (name, package) in registry.packages().iter() {
            let (desc, url, lang) = (
                package.description,
                package.url,
                package.language.to_string(),
            );
            if let Some(filter) = &filter {
                if !name.contains(filter) {
                    continue;
                }
            }
            eprintln!(
                "[{}] {} - {} ({}) [{} (not always accurate)]",
                "package".bold().bright_cyan(),
                name.italic().white(),
                desc.blue().bold(),
                url.purple(),
                lang.italic()
            );
        }

        return;
    }

    if let Some(package) = registry.get(&first_arg) {
        // in this case we can just assume the URL is correct.
        let url = Url::parse(package.url).unwrap_or_else(|err| {
            panic!(
                "the internal package registry contained an invalid URL. This is a bug. Url={} Msg={}",
                package.url, err
            );
        });

        let _ = match Installer::new(&url) {
            Ok(i) => i,
            Err(e) => {
                let message = e.to_string();
                outputln!(red, "failed to install package. {}", message);
                return;
            }
        };

        outputln!(green, "successfully installed package `{}`", first_arg);
        return;
    }

    let link = &first_arg;

    let url = match Url::parse(link) {
        Ok(url) => url,
        Err(e) => {
            usage(
                &program_name,
                Some(format!(
                    "invalid argument (expect package-name/url): {} ({})",
                    e, link
                )),
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
        "note: use `sudo rm -rf /tmp/cinstall-*` to remove any temporary directories."
    );
}
