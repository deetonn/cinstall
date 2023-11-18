# cinstall
Install any C/C++ project locally using one command. The only requirement (FOR NOW) is that it's a cmake project.
You simply run `cinstall github-link-to-repo` and the rest is done for you!

# Usage

Let's say you want to install a library nice and quickly, for this example we will use [fmt](https://github.com/fmtlib/fmt). 

All you have to do is run `cinstall https://github.com/fmtlib/fmt`

This will `git clone` the project into a temp directory, run `cmake` and then run `make install`.

If there are dependencies that aren't installed such as `make` or `cmake`, you will be prompted to install them and we
automatically install them using your package manager.
