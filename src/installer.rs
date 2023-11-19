use crate::{output, outputln};
use colored::Colorize;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::io::Write;
use std::path::PathBuf;
use std::{
    io::Error,
    path::Path,
    process::{Command, ExitStatus},
};
use url::Url;

pub enum InstallError {
    DeniedInstall,
    UnknownPackageManager,
    InstallError,
    CouldNotStartProcess(String),
    FailedToClone,
    CMakeFailed,
    FailedToCreateDirectory,
    FailedToMakeInstall,
    FailedToChangeDirectory,
    BadDirectory(String),
    FailedToWriteToFile,
    UnknownFatal(String),
}

impl ToString for InstallError {
    fn to_string(&self) -> String {
        type E = InstallError;
        match self {
            E::DeniedInstall => "user denied the install of required dependencies.".into(),
            E::UnknownPackageManager => "this system uses an unknown package manager, please install git, cmake and make manually.".into(),
            E::InstallError => "failed to execute a critical operation. (this usually means we failed to start a subcommand like git or cmake)".into(),
            E::CouldNotStartProcess(process) => format!("failed to start the program `{}`", process),
            E::FailedToClone => "failed to clone the specified repository.".into(),
            E::CMakeFailed => "cmake failed to generated the projects makefile.".into(),
            E::FailedToCreateDirectory => "failed to create temporary directory to build the project from.".into(),
            E::BadDirectory(path) => format!("we were supplied a bad directory: `{}`", path),
            E::FailedToMakeInstall => "`make install` failed.".into(),
            E::FailedToChangeDirectory => "failed to set the environment directory. (this is a bizzare error)".into(),
            E::FailedToWriteToFile => "failed to write to a file when installing the package.".into(),
            E::UnknownFatal(message) => message.clone()
        }
    }
}

pub fn ask_to_install(program: &str) -> Result<(), InstallError> {
    outputln!(
        "the program `{}` is required to install this package.",
        program
    );
    output!("install it now? [Y/n] ");
    let input: String = text_io::read!("{}");

    if input.is_empty() {
        outputln!(purple, "nothing entered, assuming you meant no.");
        return Err(InstallError::DeniedInstall);
    }

    if input.to_lowercase().chars().next().unwrap_or('n') != 'y' {
        outputln!("okay, skipping installation.");
        return Err(InstallError::DeniedInstall);
    }

    let status: Result<ExitStatus, Error>;

    if Path::new("/usr/bin/pacman").exists() {
        status = Command::new("sudo")
            .arg("pacman")
            .arg("-S")
            .arg(program)
            .status();
    } else if Path::new("/usr/bin/apt").exists() {
        status = Command::new("sudo")
            .arg("apt")
            .arg("install")
            .arg(program)
            .status();
    } else {
        return Err(InstallError::UnknownPackageManager);
    }

    match status {
        Ok(exit_status) => {
            if !exit_status.success() {
                outputln!(red, "package manager failed to install required package.");
                return Err(InstallError::InstallError);
            }
            Ok(())
        }
        Err(e) => {
            outputln!(red, "failed to execute program: {}", e);
            Err(InstallError::InstallError)
        }
    }
}

// make sure they have CMake and git.
pub fn verify_has_programs() -> Result<(), InstallError> {
    if !Path::new("/usr/bin/git").exists() {
        ask_to_install("git")?;
    }

    if !Path::new("/usr/bin/cmake").exists() {
        ask_to_install("cmake")?;
    }

    if !Path::new("/usr/bin/make").exists() {
        ask_to_install("make")?;
    }

    eprintln!("user has all required dependencies.");
    Ok(())
}

pub enum InstallMethod {
    RunCMake,
    MakeInstall,
    MoveHeaders(Vec<String>),
    Unknown(String),
}

macro_rules! with_temp_path {
    ($path:ident, $body:block) => {{
        let old_path = match std::env::current_dir() {
            Ok(p) => p,
            Err(e) => {
                return Err(InstallError::UnknownFatal(format!("failed to temporarily switch to temp directory. {}", e.to_string())));
            }
        };

        match std::env::set_current_dir($path) {
            Ok(_) => (),
            Err(_) => {
                return Err(InstallError::FailedToChangeDirectory);
            }
        };

        $body

        match std::env::set_current_dir(&old_path) {
            Ok(_) => (),
            Err(_) => {
                return Err(InstallError::UnknownFatal("failed to switch directory back to original.".into()));
            }
        };
    }};
}

pub fn resolve_makefile_install_method(path: &Path) -> Result<InstallMethod, InstallError> {
    outputln!(
        green,
        "checking what install methods are available in the makefile."
    );

    let file_contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return Err(InstallError::UnknownFatal(format!(
                "failed to read makefile. {}",
                e
            )));
        }
    };

    let file_contents: Vec<String> = file_contents.split('\n').map(String::from).collect();

    // We need to check for the rule: `install:`
    let has_install = file_contents.iter().any(|item| &**item == "install:");

    // There is no install procedure available.
    if has_install {
        Ok(InstallMethod::MakeInstall)
    } else {
        Err(InstallError::UnknownFatal(
            "the makefile has no `install` procedure.".into(),
        ))
    }
}

pub fn execute_cmake(path: &Path) -> Result<(), InstallError> {
    with_temp_path!(path, {
        let result = Command::new("cmake").arg(".").status();

        match result {
            Ok(status) => {
                if !status.success() {
                    return Err(InstallError::CMakeFailed);
                }
                outputln!(green, "cmake was successful");
            }
            Err(e) => {
                return Err(InstallError::CouldNotStartProcess(format!(
                    "failed to start cmake: {}",
                    e
                )))
            }
        }
    });

    Ok(())
}

pub fn execute_make_custom(path: &Path) -> Result<(), InstallError> {
    // `make install` failed, we run `make help` to try and output information about the Makefile
    // and then prompt the user to input arguments.
    //
    with_temp_path!(path, {
        let make_help_status = Command::new("make").arg("help").status();

        if make_help_status.is_err() {
            outputln!("failed to output help information, you are on your own here...");
            let tmp_path = path.to_str().unwrap();
            outputln!(
                "to help follow along with the next part, please go to {}/Makefile",
                tmp_path
            );
        }

        let mut option = String::new();
        let mut done = false;

        outputln!(green, "enter `stop` to exit this prompt.");

        while !done {
            option.clear();
            output!(on_blue, "please enter a build option: ");
            option = text_io::read!("{}\n");

            if option == "stop" {
                done = true;
                continue;
            }

            let current_command_exec = Command::new("make").arg(&option).status();
            match current_command_exec {
                Ok(result) => {
                    if !result.success() {
                        outputln!("that didn't quite work, try again.");
                        continue;
                    }
                    done = true;
                    outputln!("success! hopefully it is all installed now.");
                    continue;
                }
                Err(e) => {
                    outputln!("something went wrong on our end... sorry.");
                    outputln!("reason: {}", e);
                    continue;
                }
            }
        }
    });

    Ok(())
}

pub fn execute_make_install(path: &Path) -> Result<(), InstallError> {
    with_temp_path!(path, {
        let status = Command::new("make").arg("install").status();

        match status {
            Ok(result) => {
                if !result.success() {
                    return execute_make_custom(path);
                }
                outputln!("`make install` was successful!");
            }
            Err(e) => {
                return Err(InstallError::CouldNotStartProcess(e.to_string()));
            }
        }
    });

    Ok(())
}

pub fn try_get_install_headers(path: &Path) -> Result<InstallMethod, InstallError> {
    let mut files = vec![];
    with_temp_path!(path, {
        let _ = Command::new("ls").status();
        let mut running = true;

        outputln!("enter `stop` to close this prompt and continue.");
        outputln!("please select headers you'd like to install.");
        while running {
            output!(green, "name: ");
            let input: String = text_io::read!("{}\n");

            if input == "stop" {
                running = false;
                continue;
            }

            files.push(input);
        }
    });

    let full_paths_to_files: Vec<String> = files
        .iter()
        .map(|header_file| {
            let mut buf = PathBuf::new();
            buf.push(path);
            buf.push(header_file);

            if !buf.as_path().exists() {
                let faulty_path = buf.as_path().to_str().unwrap();
                outputln!(red, "the file `{}` does not exist.", faulty_path);
                outputln!(red, "it will be skipped during moving of files.");
            }

            buf.as_path().to_str().unwrap().to_string()
        })
        .collect();

    Ok(InstallMethod::MoveHeaders(full_paths_to_files))
}

pub fn resolve_install_method(path: &Path) -> InstallMethod {
    // We need to check if the "Makefile" has an install
    // section
    let mut path_to_makefile = PathBuf::from(path);
    path_to_makefile.push("Makefile");

    if path_to_makefile.as_path().exists() {
        match resolve_makefile_install_method(path) {
            Ok(method) => return method,
            Err(e) => {
                outputln!("cannot install using make, there is no install routine.");
                return InstallMethod::Unknown(e.to_string());
            }
        }
    }

    let mut path_to_makefile = PathBuf::from(path);
    path_to_makefile.push("CMakeLists.txt");

    if path_to_makefile.exists() {
        // NOTE: This is a pre-step. After running cmake,
        //       the Make path with of course be hit.
        return InstallMethod::RunCMake;
    }

    match try_get_install_headers(path) {
        Ok(m) => m,
        Err(e) => InstallMethod::Unknown(e.to_string()),
    }
}

pub fn move_file(src: &Path, dest: &Path) -> Result<(), InstallError> {
    let destination = dest.to_str().unwrap_or("<destination path>");
    let source = src.to_str().unwrap_or("<source path>");

    outputln!(green, "moving `{}` to `{}`", source, destination);

    let mut file = match std::fs::File::create(destination) {
        Ok(f) => f,
        Err(e) => {
            return Err(InstallError::BadDirectory(format!(
                "{}: {} (you may need to `sudo`)",
                destination, e
            )));
        }
    };

    let source_contents = std::fs::read_to_string(src)
        .map_err(|item| InstallError::UnknownFatal(item.to_string()))?;

    write!(file, "{}", source_contents).map_err(|_| InstallError::FailedToWriteToFile)?;

    Ok(())
}

pub fn execute_install_headers(headers: &[String]) -> Result<(), InstallError> {
    // headers must be moved into /usr/local/include/
    const ROOT_PATH: &str = "/usr/local/include/";
    for item in headers.iter() {
        let file_name = match item.split('/').last() {
            Some(last) => last,
            None => {
                outputln!("failed to get file name for path {}.", item);
                continue;
            }
        };
        let buf: PathBuf = [ROOT_PATH, file_name].iter().collect();
        let from = Path::new(item);
        let to = buf.as_path();

        move_file(from, to)?;
    }
    Ok(())
}

pub fn execute_install_method(path: &Path, method: &InstallMethod) -> Result<(), InstallError> {
    match method {
        InstallMethod::Unknown(message) => Err(InstallError::UnknownFatal(message.clone())),
        InstallMethod::RunCMake => execute_cmake(path),
        InstallMethod::MoveHeaders(headers) => execute_install_headers(headers),
        InstallMethod::MakeInstall => execute_make_install(path),
    }
}

pub struct Installer {
    path: String,
}

impl Installer {
    pub fn new(url: &Url) -> Result<Self, InstallError> {
        verify_has_programs()?;
        let random_tag: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        let temp_path = format!("/tmp/cinstall-{}", random_tag);

        if !Path::new(&temp_path).exists() {
            match std::fs::create_dir_all(&temp_path) {
                Ok(_) => (),
                Err(e) => {
                    outputln!(
                        red,
                        "failed to create temporary directory for git repository."
                    );
                    outputln!(red, "reason: {}", e);
                    return Err(InstallError::FailedToCreateDirectory);
                }
            }
        }

        // clone the project to our temporary path.
        match Command::new("git")
            .arg("clone")
            .arg(url.to_string())
            .arg(&temp_path)
            .status()
        {
            Ok(status) => {
                if !status.success() {
                    let code = status.code().unwrap_or(-1);
                    outputln!(
                        red,
                        "failed to git clone to repository (exited with code {})",
                        code
                    );
                    return Err(InstallError::FailedToClone);
                }
                outputln!(green, "cloned project to {}", temp_path);
            }
            Err(e) => {
                outputln!(red, "failed to clone: {}", e);
                return Err(InstallError::CouldNotStartProcess("git".into()));
            }
        };

        // use cmake to build a Makefile
        let path = Path::new(&temp_path);
        let method = resolve_install_method(path);

        if let InstallMethod::Unknown(message) = &method {
            return Err(InstallError::UnknownFatal(message.clone()));
        }

        match execute_install_method(path, &method) {
            Ok(_) => outputln!("all execution steps completed successfully."),
            Err(e) => {
                return Err(e);
            }
        };

        // execute make after we have ran cmake.
        if let InstallMethod::RunCMake = method {
            execute_make_install(path)?;
        }

        Ok(Self { path: temp_path })
    }

    pub fn temp_path(&self) -> &String {
        &self.path
    }
}
