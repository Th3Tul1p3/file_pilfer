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
use ntfs::NtfsFile;
use ntfs_colin_finck::{cd, cd_root, get /* , ls*/};
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
}

fn main() -> Result<()> {
    // dump a whole directory
    // dump a list of file from a text file
    let mut path: String = "".to_string();

    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("The goal is to be able to copy protected file in windows.");
        ap.refer(&mut path)
            .add_option(&["--path"], Store, "PATH path into drive (Defautlt C:)");
        ap.parse_args_or_exit();
    }

    let drive: String = r"\\.\C:".to_string();

    // read file system
    let f: File = File::open(&drive).unwrap();
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

    if cd(&path, &mut info) == "" {
        println!("Error when changing Directory.");
        process::exit(0x0100);
    }
    let collect_path_parts = path.split("\\").collect::<Vec<&str>>();
    let target_filename = collect_path_parts.last().unwrap();
    let _res = get(target_filename, &mut info, ".");
    //ls(&mut info);
    cd_root(&mut info);
    println!();
    Ok(())
}
