# Source in Git Bash to use the Rust toolchain kept in C:\RUST: . tools/env.sh
export RUSTUP_HOME="C:/RUST/.toolchain/rustup"
export CARGO_HOME="C:/RUST/.toolchain/cargo"
# C:\RUST\.toolchain\llvm-dlltool holds llvm-ar from the llvm-tools component
# under the names dlltool.exe and llvm-dlltool.exe (it is the same program).
# rustc needs one to make import libraries for windows-sys.
export PATH="/c/RUST/.toolchain/cargo/bin:/c/RUST/.toolchain/llvm-dlltool:/c/RUST/.toolchain/rustup/toolchains/1.99.1-x86_64-pc-windows-gnu/lib/rustlib/x86_64-pc-windows-gnu/bin/self-contained:$PATH"
