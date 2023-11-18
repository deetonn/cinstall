use crate::{output, outputln};
use colored::Colorize;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
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
            E::FailedToChangeDirectory => "failed to set the environment directory. (this is a bizzare error)".into()
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

        let temp_path = format!("/tmp/cppinstall-{}", random_tag);

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

        // set the current working directory so cmake and make dont shit themselves.
        match std::env::set_current_dir(&temp_path) {
            Ok(_) => (),
            Err(e) => {
                outputln!(red, "failed to set active directory.");
                outputln!(red, "reason: {}", e);
                return Err(InstallError::FailedToChangeDirectory);
            }
        };

        // use cmake to build a makefile
        match Command::new("cmake").arg(&temp_path).status() {
            Ok(status) => {
                let code = status.code().unwrap_or(-1);
                if !status.success() {
                    outputln!(
                        red,
                        "cmake failed to generated makefile. (exitied with code {})",
                        code
                    );
                    return Err(InstallError::CMakeFailed);
                }
                outputln!(green, "cmake has finished generated makefile.");
            }
            Err(e) => {
                outputln!(red, "failed to run cmake: {}", e);
                return Err(InstallError::CouldNotStartProcess("cmake".into()));
            }
        };

        // use make to install the project locally.
        match Command::new("make").arg(&temp_path).arg("install").status() {
            Ok(status) => {
                if !status.success() {
                    let code = status.code().unwrap_or(-1);
                    outputln!(
                        red,
                        "failed to `make install` the project. (exited with code {})",
                        code
                    );
                    return Err(InstallError::FailedToMakeInstall);
                }
                outputln!(green, "successfully installed project locally!");
            }
            Err(e) => {
                outputln!(red, "failed to run `make install`: {}", e);
                return Err(InstallError::CouldNotStartProcess("make".into()));
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
