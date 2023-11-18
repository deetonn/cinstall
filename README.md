# cinstall
Install any C/C++ project locally using one command. The only requirement (**FOR NOW**) is that it's a cmake/make project.
You simply run `cinstall package-name | package-url` and the rest is done for you!

# Usage

Let's say you want to install a library nice and quickly, for this example we will use [fmt](https://github.com/fmtlib/fmt). 

## Commandline
* `cinstall --list-packages` -- This lists all packages.
* `cinstall --list-packages json` -- Lists all packages that have `json` in their name.
* `cinstall {fmt}` -- Will install the package mentioned above.

All you have to do is run `cinstall https://github.com/fmtlib/fmt`

This will `git clone` the project into a temp directory, run `cmake` and then run `make install`.

If there are dependencies that aren't installed such as `make` or `cmake`, you will be prompted to install them and we
automatically install them using your package manager.

# Packages

Packages are generated using a python script. This script is located at the root of this project called
`scrape_project_info.py`. This will take a giant list of known C++ library's and turn them into some 
json that `serde_json` can parse into a `HashMap<&str, Package>`.

I plan on having it filter things out that are not needed, and in the future we can also add more
packages to it. This solution means the packages can change without an update.

This list is inserted into the programs source at compile-time using `include_str!()`.
