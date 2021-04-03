use std::{process::Command, str::from_utf8};

use assert_cmd::prelude::*;
use itertools::Itertools;

use crate::ErrConversion;

use super::MockFs;

pub fn zr(args: &[&str]) {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .map(|mut it| it.args(args).current_dir(MockFs::home()).assert().success()).unwrap();
}

pub fn zr_output(args: &[&str]) -> String {
    let stdout = Command::cargo_bin(env!("CARGO_PKG_NAME")).wrap()
        .and_then(|mut it| it.args(args).current_dir(MockFs::home()).output().wrap())
        .unwrap()
        .stdout;
    std::str::from_utf8(stdout.as_slice()).unwrap().to_string()
}

pub fn zr_dbg(args: &[&str]) {
    let output = Command::cargo_bin(env!("CARGO_PKG_NAME")).wrap()
        .and_then(|mut it| it
            .arg("--debug")
            .args(args)
            .current_dir(MockFs::home())
            .output()
            .wrap()
        ).unwrap();
    println!("status: {:?}", output.status.code());
    println!("stdout: {:#?}", from_utf8(output.stdout.as_slice()).unwrap().split("\n").collect_vec());
    println!("stderr: {:#?}", from_utf8(output.stderr.as_slice()).unwrap().split("\n").collect_vec());
}

pub fn zr_fail(args: &[&str]) {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .map(|mut it| it.args(args).current_dir(MockFs::home()).assert().failure()).unwrap();
}
