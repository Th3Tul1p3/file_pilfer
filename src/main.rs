// Based on the code ntfs shell from Colin Finck <colin@reactos.org>
// modified to cd a complete path and not just a directory by directory
// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::{bail, Context, Result};
use ntfs::indexes::NtfsFileNameIndex;
use ntfs::Ntfs;
use std::env;
use std::fs::File;
mod sector_reader;
use ntfs::NtfsFile;
use sector_reader::SectorReader;
use std::fs::OpenOptions;
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
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: exe PATH");
        eprintln!();
        eprintln!("PATH path into C: drive");
        bail!("Aborted");
    }

    // read file system
    let path: String = r"\\.\C:".to_string();
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

    let _result: bool = cd(&args[1], &mut info);
    let _result = get(&args[1], &mut info);
    ls(&mut info);
    cd_root(&mut info);
    Ok(())
}

fn cd<T>(arg: &str, info: &mut CommandInfo<T>) -> bool
where
    T: Read + Seek,
{
    let dir_list_from_input = arg.split(r"\");
    for dir in dir_list_from_input.into_iter() {
        if dir == ".." {
            info.current_directory.pop();
        } else {
            let index = match info
                .current_directory
                .last()
                .unwrap()
                .directory_index(&mut info.fs)
            {
                Ok(index) => index,
                Err(_) => return false,
            };
            let mut finder = index.finder();
            let maybe_entry = NtfsFileNameIndex::find(&mut finder, info.ntfs, &mut info.fs, dir);

            if maybe_entry.is_none() {
                println!(
                    "Cannot find subdirectory \"{dir}\".\nStop at : {}",
                    info.current_directory_name
                );
                return false;
            }

            let entry = match maybe_entry.unwrap() {
                Ok(entry) => entry,
                Err(_) => return false,
            };
            let file_name = match entry.key().expect("key must exist for a found Index Entry") {
                Ok(file_name) => file_name,
                Err(_) => return false,
            };

            if file_name.is_directory() {
                println!("\"{dir}\" is not a file.");
                return false;
            }

            let file = match entry.to_file(info.ntfs, &mut info.fs) {
                Ok(file) => file,
                Err(_) => return false,
            };
            info.current_directory_name += &format!("{}\\", dir);
            info.current_directory.push(file);
        }
    }
    return true;
}

fn cd_root<T>(info: &mut CommandInfo<T>)
where
    T: Read + Seek,
{
    info.current_directory = vec![info.ntfs.root_directory(&mut info.fs).unwrap()];
    info.current_directory_name = String::from(r"C:\");
}

fn ls<T>(info: &mut CommandInfo<T>)
where
    T: Read + Seek,
{
    // for debug purpose
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
}

fn get<T>(arg: &str, _info: &mut CommandInfo<T>) -> Result<()>
where
    T: Read + Seek,
{
    // Extract any specific $DATA stream name from the file.
    let (file_name, data_stream_name) = match arg.find(':') {
        Some(mid) => (&arg[..mid], &arg[mid + 1..]),
        None => (arg, ""),
    };

    // Compose the output file name and try to create it.
    // It must not yet exist, as we don't want to accidentally overwrite things.
    let output_file_name = if data_stream_name.is_empty() {
        file_name.to_string()
    } else {
        format!("{file_name}_{data_stream_name}")
    };
    let mut _output_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output_file_name)
        .with_context(|| format!("Tried to open \"{output_file_name}\" for writing"))?;

    // Open the desired file and find the $DATA attribute we are looking for.
    /*let file = parse_file_arg(file_name, info)?;
    let data_item = match file.data(&mut info.fs, data_stream_name) {
        Some(data_item) => data_item,
        None => {
            println!("The file does not have a \"{data_stream_name}\" $DATA attribute.");
            return Ok(());
        }
    };
    let data_item = data_item?;
    let data_attribute = data_item.to_attribute()?;
    let mut data_value = data_attribute.value(&mut info.fs)?;

    println!(
        "Saving {} bytes of data in \"{}\"...",
        data_value.len(),
        output_file_name
    );
    let mut buf = [0u8; 4096];

    loop {
        let bytes_read = data_value.read(&mut info.fs, &mut buf)?;
        if bytes_read == 0 {
            break;
        }

        output_file.write_all(&buf[..bytes_read])?;
    }*/

    Ok(())
}