### You may find the following cargo features handy:

* cargo init creates a new project. That's what you used to make the hello world program. If you really don't want to be using git, you can type cargo init --vcs none (projectname).
* cargo build downloads all dependencies for a project and compiles them, and then compiles your program. It doesn't actually run your program - but this is a good way to quickly find compiler errors.
* cargo update will fetch new versions of the crates you listed in your cargo.toml file (see below).
* cargo clean can be used to delete all of the intermediate work files for your project, freeing up a bunch of disk space. They will automatically download and recompile the next time you run/build your project. Occasionally, a cargo clean can help when things aren't working properly - particularly IDE integration.
* cargo verify-project will tell you if your Cargo settings are correct.
* cargo install can be used to install programs via Cargo. This is helpful for installing tools that you need.


### Cargo also supports extensions - that is, plugins that make it do even more. There are some that you may find particularly useful:

* Cargo can reformat all your source code to look like standard Rust from the Rust manuals. You need to type rustup component add rustfmt once to install the tool. After that's done, you can type cargo fmt to format your code at any time.

* If you'd like to work with the mdbook format - used for this book! - cargo can help with that, too. Just once, you need to run cargo install mdbook to add the tools to your system. After that, mdbook build will build a book project, mdbook init will make a new one, and mdbook serve will give you a local webserver to view your work! You can learn all about mdbook on their documentation page.
* Cargo can also integrate with a "linter" - called Clippy. Clippy is a little pedantic (just like his Microsoft Office namesake!). Just the once, run rustup component add clippy. You can now type cargo clippy at any time to see suggestions for what may be wrong with your code!