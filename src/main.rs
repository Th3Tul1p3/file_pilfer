// Based on the code ntfs shell from Colin Finck <colin@reactos.org>
// modified to cd a complete path and not just a directory by directory
// SPDX-License-Identifier: MIT OR Apache-2.0

//use argparse::{ArgumentParser, StoreTrue, Store};
use anyhow::{bail, Result};
use ntfs::Ntfs;
use std::env;
use std::fs::File;
mod sector_reader;
use sector_reader::SectorReader;
mod ntfs_colin_finck;
use ntfs::NtfsFile;
use ntfs_colin_finck::{cd, cd_root, get, ls};
use std::io::{BufReader, Read, Seek};

pub struct CommandInfo<'n, T>
where
    T: Read + Seek,
{
    current_directory: Vec<NtfsFile<'n>>,
    current_directory_name: String,
    fs: T,
    ntfs: &'n Ntfs,
}

fn main() -> Result<()> {
    let path: String = r"\\.\C:".to_string();
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: exe PATH");
        eprintln!();
        eprintln!("PATH path into C: drive");
        bail!("Aborted");
    }

    // read file system
    let f: File = File::open(&path).unwrap();
    let sr: SectorReader<File> = SectorReader::new(f, 4096).unwrap();
    let mut fs: BufReader<SectorReader<File>> = BufReader::new(sr);
    let mut ntfs: Ntfs = Ntfs::new(&mut fs).unwrap();
    ntfs.read_upcase_table(&mut fs).unwrap();

    // initialize with the content of first directory
    let current_directory: Vec<NtfsFile> = vec![ntfs.root_directory(&mut fs).unwrap()];
    let mut info: CommandInfo<BufReader<SectorReader<File>>> = CommandInfo {
        current_directory,
        current_directory_name: String::from(r"C:\"),
        fs,
        ntfs: &ntfs,
    };

    let result = cd(&args[1], &mut info);
    println!("The file you want: {result}");
    let collect_path_parts = &args[1].split("\\").collect::<Vec<&str>>();
    let target_filename = collect_path_parts.last().unwrap();
    println!("argument : {:?}", target_filename);
    let result_get = get(target_filename, &mut info, "..\\dump");
    println!("{:?}", result_get);
    ls(&mut info);
    cd_root(&mut info);
    println!();
    Ok(())
}
