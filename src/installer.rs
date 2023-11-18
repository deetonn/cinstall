use crate::{output, outputln};
use colored::Colorize;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::io::Read;
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
            E::FailedToMakeInstall => "`make install` failed.".into(),
            E::FailedToChangeDirectory => "failed to set the environment directory. (this is a bizzare error)".into(),
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

pub fn execute_make_install(path: &Path) -> Result<(), InstallError> {
    with_temp_path!(path, {
        let status = Command::new("make").arg("install").status();

        match status {
            Ok(result) => {
                if !result.success() {
                    return Err(InstallError::FailedToMakeInstall);
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

pub fn resolve_install_method(path: &Path) -> InstallMethod {
    if path.join("/Makefile").exists() {
        // We need to check if the "Makefile" has an install
        // method.
    }

    if path.join("/CMakeLists.txt").exists() {
        // NOTE: This is a pre-step. After running cmake,
        //       the Make path with of course be hit.
        return InstallMethod::RunCMake;
    }

    InstallMethod::Unknown(
        "this repository has no known way of installation. (cmake and make was tried)".into(),
    )
}

pub fn execute_install_method(path: &Path, method: InstallMethod) -> Result<(), InstallError> {
    match method {
        InstallMethod::Unknown(message) => Err(InstallError::UnknownFatal(message)),
        InstallMethod::RunCMake => execute_cmake(path),
        InstallMethod::MakeInstall => execute_make_install(path),
    }
}

pub struct Installer<'a> {
    url: &'a Url,
    path: String,
}

impl<'a> Installer<'a> {
    pub fn new(url: &'a Url) -> Result<Self, InstallError> {
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

        match execute_install_method(path, method) {
            Ok(_) => outputln!("all execution steps completed successfully."),
            Err(e) => {
                return Err(e);
            }
        };

        Ok(Self {
            url,
            path: temp_path,
        })
    }

    pub fn temp_path(&self) -> &String {
        &self.path
    }
}
