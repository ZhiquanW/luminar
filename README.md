# laminar

```Manage computing resources for multiple users on single machine.```

NOTE: We only support NVIDIA GPU currently, AMD GPU is not yet supported.

The current version v0.1.3 only supports monitoring of CPU & GPU usage for each user. The process management is disabled for further development.
# Usage
```
$ luminar server .luminar
```
This command runs $lumianr$ in the current directiory and stores all the log files in $.lumianr$

# Installation
This utility is available via [crate.io](https://crates.io/crates/luminar)
```
$ cargo install luminar
```

