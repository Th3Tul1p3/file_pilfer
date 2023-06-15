// Based on the code ntfs shell from Colin Finck <colin@reactos.org>
// modified to cd a complete path and not just a directory by directory
// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use argparse::{ArgumentParser, Store};
use ntfs::Ntfs;
use std::fs::File;
mod sector_reader;
use sector_reader::SectorReader;
mod ntfs_colin_finck;
use argparse::List;
use ntfs::NtfsFile;
use ntfs_colin_finck::{cd, cd_root, get /* , ls*/};
use std::fs;
use std::io::{BufReader, Read, Seek};
use std::process;

pub struct CommandInfo<'n, T>
where
    T: Read + Seek,
{
    current_directory: Vec<NtfsFile<'n>>,
    current_directory_name: String,
    fs: T,
    ntfs: &'n Ntfs,
    output: String,
}

fn main() -> Result<()> {
    // dump a whole directory
    let mut paths: Vec<String> = Vec::new();
    let mut output: String = ".".to_string();
    let drive: String = r"\\.\C:".to_string();
    let mut input_file: String = "".to_string();
    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("The goal is to be able to copy protected file in windows.");
        ap.refer(&mut paths)
            .add_option(&["-p"], List, "PATH path into drive. (Defautlt from C:)");
        ap.refer(&mut output).add_option(
            &["-o"],
            Store,
            "Output directory for downloaded files.  (Defautlt '.')",
        );
        ap.refer(&mut input_file)
            .add_option(&["-f"], Store, "Input multiple file too gather.");
        ap.parse_args_or_exit();
    }

    if output != "." {
        // might throws an error if it's not possible to create
        fs::create_dir_all(&output).unwrap();
    }

    // read file system
    let f: File = File::open(&drive).unwrap();
    let sr: SectorReader<File> = SectorReader::new(f, 4096).unwrap();
    let mut fs: BufReader<SectorReader<File>> = BufReader::new(sr);
    let mut ntfs: Ntfs = Ntfs::new(&mut fs).unwrap();
    ntfs.read_upcase_table(&mut fs).unwrap();

    // initialize with the content of first directory;
    let mut info: CommandInfo<BufReader<SectorReader<File>>> = CommandInfo {
        current_directory: vec![ntfs.root_directory(&mut fs).unwrap()],
        current_directory_name: String::from(r"C:\"),
        fs,
        ntfs: &ntfs,
        output,
    };

    if paths.len() != 0 && input_file != "" {
        eprintln!("You must choose between -p and -f option.");
        process::exit(0x0100);
    } else if input_file != "" {
        paths = read_input_file(input_file);
    }

    for mut path in paths.into_iter() {
        println!("{}", path);
        if path.contains("\r"){
            path = path.replace("\r", "");
        }
        cd_get_cd_dot_dot(path, &mut info);
    }

    Ok(())
}

fn cd_get_cd_dot_dot<'n, T>(path: String, info: &mut CommandInfo<'n, T>)
where
    T: Read + Seek,
{
    if cd(&path, info) == "" {
        eprintln!("Error when changing Directory.");
        process::exit(0x0100);
    }

    let collect_path_parts = path.split("\\").collect::<Vec<&str>>();
    let target_filename = collect_path_parts.last().unwrap();
    let _res = get(target_filename, info);
    cd_root(info);
}

fn read_input_file(input_file: String) -> Vec<String> {
    let output_file =
        fs::read_to_string(input_file).expect("Should have been able to read the file");
    output_file
        .split('\n')
        .map(|borrowed_str| borrowed_str.to_string())
        .collect::<Vec<String>>()
}
