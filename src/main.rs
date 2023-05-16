// Based on the code from Colin Finck <colin@reactos.org>
// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use ntfs::indexes::NtfsFileNameIndex;
use ntfs::{Ntfs, NtfsError};
use std::fs::File;
mod sector_reader;
use ntfs::NtfsFile;
use sector_reader::SectorReader;
use std::io::{BufReader, Read, Seek};


struct CommandInfo<'n, T>
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
    let f = File::open(&path).unwrap();
    let sr = SectorReader::new(f, 4096).unwrap();
    let mut fs: BufReader<SectorReader<File>> = BufReader::new(sr);
    let mut ntfs = Ntfs::new(&mut fs).unwrap();
    ntfs.read_upcase_table(&mut fs).unwrap();

    let current_directory = vec![ntfs.root_directory(&mut fs).unwrap()];

    let mut info = CommandInfo {
        current_directory,
        current_directory_name : String::from("C:\\"),
        fs,
        ntfs: &ntfs,
    };

    println!("{:?}", info.current_directory.len());

    let path = r"Users\joseph";
    let _result = cd(path, &mut info);

    info.current_directory = vec![info.ntfs.root_directory(&mut info.fs).unwrap()];

    let index = info
        .current_directory
        .last()
        .unwrap()
        .directory_index(&mut info.fs)
        .unwrap();
    let mut iter = index.entries();

    while let Some(entry) = iter.next(&mut info.fs) {
        let entry = entry.unwrap();
        let file_name = entry
            .key()
            .expect("key must exist for a found Index Entry")
            .unwrap();

        let prefix = if file_name.is_directory() {
            "<DIR>"
        } else {
            ""
        };
        println!("{:5}  {}", prefix, file_name.name());
    }
    Ok(())
}

fn cd<T>(arg: &str, info: &mut CommandInfo<T>) -> Result<(), NtfsError>
where
    T: Read + Seek,
{
    if arg.is_empty() {
        return Ok(());
    }

    let dir_list_from_input = arg.split(r"\");
    for dir in dir_list_from_input.into_iter() {
        if dir == ".." {
            info.current_directory.pop();
        } else {
            let index = info
                .current_directory
                .last()
                .unwrap()
                .directory_index(&mut info.fs)?;
            let mut finder = index.finder();
            let maybe_entry = NtfsFileNameIndex::find(&mut finder, info.ntfs, &mut info.fs, dir);

            if maybe_entry.is_none() {
                println!("Cannot find subdirectory \"{dir}\".\nStop at : {}", info.current_directory_name);
                return Ok(());
            }else {
                info.current_directory_name += &format!("{}\\",dir);
            }

            let entry = maybe_entry.unwrap()?;
            let file_name = entry
                .key()
                .expect("key must exist for a found Index Entry")?;

            if !file_name.is_directory() {
                println!("\"{dir}\" is not a directory.");
                return Ok(());
            }

            let file = entry.to_file(info.ntfs, &mut info.fs)?;

            info.current_directory.push(file);
        }
    }
    Ok(())
}
