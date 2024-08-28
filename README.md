## How to Use:

To compile and run the application, you need to have Rust and Cargo installed.  
This can be done using Rustup: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

To compile and run, use: `cargo run --release`.  
It can also be compiled without the `--release` flag, but this is not recommended for CPU rendering as it is extremely slow.

You can change the rendering mode using `--mode <rendering-mode>` or `-m <rendering-mode>`.  
You can change the sample count using `--samples <sample-count>` or `-s <sample-count>`.  
You can also get help with `--help` or `-h`.  
Example: `cargo run --release -- -m multi-thread -s 256`.

It is not possible to change the resolution using the CLI.  
To change the resolution, search for the `app.rs` file and modify the `PhysicalSize` in the `WindowBuilder` within the `Application::run` function (line 439).
