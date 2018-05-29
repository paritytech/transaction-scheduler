## README

# Setup

* Install Rust
    * Note: Latest [Rust version](https://www.rust-lang.org)
    ```
    curl https://sh.rustup.rs -sSf | sh
    ```
    * Restart the terminal shell session
        ```
        source ~/.bash_profile
        ```
    * Check if `rustc` command is found
        ```
        rustc --version
        ```

* Build
    * Note: ./Cargo.toml configuration file builds source code in ./server/src and ./cli/src
subdirectories according to ./server/Cargo.toml and ./cli/Cargo.toml
configuration file's metadata and build outcome requirements.
Binary executables and dependencies are generated in the ./target folder
and Cargo.lock file is generated to track dependencies.
    ```
    cargo build --all --verbose
    ```

* Help
    ```
    ./target/debug/txsched -h
    ```

* Run
    ```
    ./target/debug/txsched
    ```

* Tests
    ```
    cargo test --all --verbose
    ```

* Docs
    ```
    open -a "Google Chrome" ./target/doc/txsched/index.html
    ```

* Deployment
    * DevOps Automation - [Ansible](https://www.ansible.com/)
    * Ethereum API - Network peering for Dapps - [Infura](https://infura.io/)
