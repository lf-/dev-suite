#!/usr/bin/env python3
import subprocess

subprocess.run("cargo build --all", shell=True, check=True)
subprocess.run("cargo test --all -- --test-threads=1", shell=True, check=True)
subprocess.run("rustup run nightly cargo fmt --all -- --check", shell=True, check=True)
subprocess.run("cargo clippy --all --all-targets -- -W clippy::pedantic", shell=True, check=True)
