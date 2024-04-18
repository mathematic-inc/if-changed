load("@crates_io//:defs.bzl", "crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_binary")

package(default_visibility = ["//visibility:public"])

rust_binary(
    name = "if-changed",
    srcs = ["if-changed.rs"],
    deps = crate_deps([
        "clap",
        "git2",
    ]),
)
