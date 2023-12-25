//  Program: sfbp
//  Author: Bill Meany
//  Date: 04/03/2020
//  Version: 1.0.0
//  Revision date: 12/14/2023
//  Revision: 1.0.8

#![allow(unused)]

//	Simple File Backup Program
//	Platform: Windows

//	Bring in code we need.

use log::info;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process;
use std::time::{Duration, Instant};

//	Bring in other crates.

use walkdir::WalkDir;

use clap::{Parser};

#[derive(Parser)]
#[command(name = "sfbp")]
#[command(author = "Bill Meany bill.meany@proton.me")]
#[command(version = "1.0.8")]
#[command(about = "Simple File Backup Program", long_about = None)]

struct Cli {
    /// Primary location for the log file, examble C:\Logs\
    log_location: String,
}

//	Get some local functions from lib.rs

use sfbp::construct_lf_name;
use sfbp::get_meta;
use sfbp::make_file_writable;
use sfbp::setup_logger;

// Define some constants

const KILO_BYTE: f64 = 1024.0;
const MEGA_BYTE: f64 = KILO_BYTE * KILO_BYTE;
const GIGA_BYTE: f64 = MEGA_BYTE * KILO_BYTE;
const NUMB_PARM: u16 = 2;
const DEBUG_FLAG: bool = false;
const RC00: i32 = 0;

//	Windows file system constants. We only make use of two of them
//	but have them all listed for documentation purposes.

const FILE_ATTRIBUTE_READONLY: u32 = 0x00000001;
const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;
const FILE_ATTRIBUTE_SYSTEM: u32 = 0x00000004;
const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x00000010;
const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x00000020;
const FILE_ATTRIBUTE_DEVICE: u32 = 0x00000040;
const FILE_ATTRIBUTE_NORMAL: u32 = 0x00000080;
const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x00000100;
const FILE_ATTRIBUTE_SPARSE_FILE: u32 = 0x00000200;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x00000400;
const FILE_ATTRIBUTE_COMPRESSED: u32 = 0x00000800;
const FILE_ATTRIBUTE_OFFLINE: u32 = 0x00001000;
const FILE_ATTRIBUTE_NOT_CONTENT_INDEXED: u32 = 0x00002000;
const FILE_ATTRIBUTE_ENCRYPTED: u32 = 0x00004000;
const FILE_ATTRIBUTE_INTEGRITY_STREAM: u32 = 0x00008000;
const FILE_ATTRIBUTE_VIRTUAL: u32 = 0x00010000;
const FILE_ATTRIBUTE_NO_SCRUB_DATA: u32 = 0x00020000;
const FILE_ATTRIBUTE_EA: u32 = 0x00040000;

//	Executable code starts here.

fn main() {
	
    info!("Beginning program execution");
	
//	Use the clap crate to parse our command line.
	
	let cli = Cli::parse();

//	All of the variable we define here are in the scope of the entire program.

    let mut log_file_name = String::with_capacity(255);
    let log_file_prefix = String::from("\\Log_");
    let mut program_name = String::with_capacity(25);
	let mut program_exe = String::with_capacity(25);
	let mut parm_file = String::with_capacity(50);
    let mut backup_source = String::new();
    let mut exclude_source = String::new();
    let mut target_base = String::new();
    let mut copy_msg_text = String::new();

    let mut bytes_copied_u64: u64 = 0;
    let bytes_copied_f64: f64 = 0.0;
    let mut display_bytes_f64: f64 = 0.0;
    let mut files_copied_f64: f64 = 0.0;
    let mut copy_new_f64: f64 = 0.0;
    let mut copy_ref_f64: f64 = 0.0;
    let mut mean_file_size_f64: f64 = 0.0;

//	Define some mutable variable we will use for file metadata.
//	We define two sets so we can perform comparisons between the
//	source and target data sets.

    let mut source_file_attrib: u32 = 0;
    let mut source_creation_time: u64 = 0;
    let mut source_access_time: u64 = 0;
    let mut source_last_write_time: u64 = 0;
    let mut source_filesize: u64 = 0;

    let mut target_file_attrib: u32 = 0;
    let mut target_creation_time: u64 = 0;
    let mut target_access_time: u64 = 0;
    let mut target_last_write_time: u64 = 0;
    let mut target_filesize: u64 = 0;

    let mut target_flag: bool = true;

    let mut bkup_master = Vec::<PathBuf>::new();
	let mut bkup_list_1 = Vec::<PathBuf>::new();
    let mut bkup_list_2 = Vec::<PathBuf>::new();
    let mut exclude_list_1 = Vec::<PathBuf>::new();

    let mut hard_drive_id = Vec::<String>::new();
    let mut hard_drive_ct = Vec::<i32>::new();

    let exe_path = env::current_exe().unwrap();
    let exe_name = exe_path.file_name().unwrap().to_str().unwrap(); 
	let (program_name, program_exe) = exe_name.split_once(".").unwrap();

//	Build the log file name using construct_lf_name from lib.rs

    construct_lf_name(&mut log_file_name, &cli.log_location, &program_name);

    setup_logger(&log_file_name, DEBUG_FLAG).expect("failed to initialize logging.");

//	Log file has been opened so we can proceed.

    info!("Beginning program execution");
    info!("File backup operation(s) initiated");
	
    let start_now = Instant::now();

//	Construct the name of our parameter file, and then create a path to it.
//	Create a code block so that objects, variable, etc. related to the
//	parameter file go away when we are done with them.

    {
        let parm_file = program_name.to_owned() + ".parms";


        info!("Attempting to open {}", parm_file);

        let fh = match File::open(&parm_file) {
            Ok(file) => file,
            Err(err) => {
                info!("{}", err);
                info!("Terminating program execution");
                process::exit(0);
            }
        };

//	If we are here then we have opened up the file.
//	Next step is to establish a handle to the file,
//	and then see if we can read the file ignoring any
//	comments in the file. A comment will start with the
//	# character.

        let pf_handle = BufReader::new(fh);

        for line in pf_handle.lines() {
			
            let line = line.expect("Unable to read line");
			
            if &line[..1] != "#" {
				
                let bkup_parms: Vec<&str> = line.split('=').collect();
				
                if bkup_parms[0].trim() == "BackupSource" {
                    backup_source = bkup_parms[1].trim().to_string();
                }
				
                if bkup_parms[0].trim() == "ExcludeSource" {
                    exclude_source = bkup_parms[1].trim().to_string();
                }
				
                if bkup_parms[0].trim() == "BackupBaseLocation" {
                    target_base = bkup_parms[1].trim().to_string();
                }
            }
			
        }
		
//	Check to be sure we have the three needed values. Error message and exit
//	if we do not.

        if backup_source.is_empty() {
            info!("No source directory list provided");
            std::process::exit(exitcode::CONFIG);
        }

        if exclude_source.is_empty() {
            info!("No exclude directory list provided");
            std::process::exit(exitcode::CONFIG);
        }

        if target_base.is_empty() {
            info!("No target directory base provided");
            std::process::exit(exitcode::CONFIG);
        }
		
		info!("Source directory list is {}", backup_source);
        info!("Exclude directory list is {}", exclude_source);
        info!("Target backup location is {}", target_base);
		
    }

//	This code block processes the source directories file.
//	We will build the list of directories into bkup_list_1.
//	We also populate hard_drive_id and hard_drive_ct for use later.

    {
        let fh = match File::open(backup_source) {
            Ok(file) => file,
            Err(err) => {
                info!("{}", err);
                info!("Terminating program execution");
                process::exit(exitcode::OK);
            }
        };

        let pf_handle = BufReader::new(fh);
        let mut source_prefix = String::with_capacity(5);

        for line in pf_handle.lines() {
            let line = line.expect("Unable to read line");

            if &line[..1] != "#" {
                source_prefix = line[0..2].to_string();

                if !hard_drive_id.contains(&source_prefix) {
                    hard_drive_id.push(source_prefix);
                    hard_drive_ct.push(0);
                }

                bkup_master.push(PathBuf::from(&line));
                info!("Base Directory = {}", &line);
            }
        }

//	Drive list will be small, but we will sort it anyway

        hard_drive_id.sort();

        let _numhard_drive_id = hard_drive_id.len();
        info!("Number of source drives is {}", _numhard_drive_id);

        let _numbkup_master = bkup_list_1.len();
        info!("Number of base directories to backup is {}", _numbkup_master);
    }

//	This code block processes the exclude directories file.
//	We will build the list of exclude directories into exclude_list_1.

    {
        let fh = match File::open(exclude_source) {
            Ok(file) => file,
            Err(err) => {
                info!("{}", err);
                info!("Terminating program execution");
                std::process::exit(exitcode::OK);
            }
        };

        let pf_handle = BufReader::new(fh);

        for line in pf_handle.lines() {
            let line = line.expect("Unable to read line");
            exclude_list_1.push(PathBuf::from(&line));
        }

        let _numexclude_list_1 = exclude_list_1.len();
        info!("Number of directories to exclude is {}", _numexclude_list_1);
    }

    {
//	Following code block obtains the metadata about the provided target
//	backup directory and validates that it is a directory.

        let mut work_path_buffer = PathBuf::new();
        work_path_buffer.push(&target_base);

        get_meta(
            &work_path_buffer,
            &mut source_file_attrib,
            &mut source_creation_time,
            &mut source_access_time,
            &mut source_last_write_time,
            &mut source_filesize,
        );

        if source_file_attrib & FILE_ATTRIBUTE_DIRECTORY == FILE_ATTRIBUTE_DIRECTORY {
            info!("{} validated as a directory structure", target_base);
        } else {
            info!("{} is not a valid directory structure!", target_base);
            info!("Terminating program execution");
            process::exit(0);
        }
    }

//
//	Next step is to build a list of all the files and directories that may
//	be candidates for a backup.
//
//	bkup_master contains the list of source directories from the source file.
//	bkup_list_1 contains the preliminary list of source directories in current directory.
//  bkup_list_2 will contain the list of all potential source directories.
//


	for current_source_entry in &bkup_master {
		
		let mut my_count: i32 = 0;
		let mut my_new_dir: i32 = 0;
		bkup_list_1.clear();
		
		bkup_list_1.push(current_source_entry.to_path_buf());
		
		{
		
        for current_source in &bkup_list_1 {
			info!("Examining the directory structure of {:?}", &current_source);
            for entry in WalkDir::new(&current_source)
                .min_depth(0)
                .sort_by(|a, b| a.file_name().cmp(b.file_name()))
            {
                match entry {
                    Ok(entry) => {
                        my_count += 1;
                        bkup_list_2.push(entry.path().to_path_buf());
                    }
                    Err(entry) => {
                        info!("Error obtaining directory entry {:?}", entry);
                    }
                };
            }
        }

        info!("Number of potential backups = {:?}", bkup_list_2.len());
		}

//	Following block removes entries from bkup_list_2 that have patterns that are
//	in exclude_list_1.
//  Clear bkup_list_1 so we can build the final list of source directories.

    bkup_list_1.clear();

    {
        let mut push_flag: bool = false;

        for entry in &bkup_list_2 {
			for x in exclude_list_1.iter().take(exclude_list_1.len()) {
	            if entry.starts_with(x) {
                    push_flag = true;
                }
            }

            if !push_flag {
                bkup_list_1.push(entry.to_path_buf());
				println!("{:?}",&entry.to_path_buf());
                push_flag = false;
            }
        }

        info!(
            "Number of potential backups after removing exclusions = {:?}",
            bkup_list_1.len()
        );
    }
	
	bkup_list_2.clear();

//	The following code block processes the entries in the bkup_list_2 vector.
//	These are all of the entries that were discovered in the previous block
//	and are either a directory or a file entry. The purpose of this block
//	is to take each entry that is a source directory and determine if the
//	associated target directory exists. To do this, we take a path entry in
//	bkup_list_2 and test to see if it is a directory. If it is, then we will
//	make a string copy of the path, strip out the colon and then prefix
//	the result with target_base to create the target path. We test to see if
//	the target exists, and if it does not we will create it.
//
//	We will use hard_drive_id[?} to increment the counts in hard_drive_ct[?].

    {

        let mut entry_length: usize = 0;
        let mut drive_count = hard_drive_id.len();
        let mut source_prefix = String::with_capacity(5);
        let mut final_path = PathBuf::new();
        let mut path_string = String::with_capacity(100);

		bkup_list_1.sort();
//		println!("bkup_list_1 size is {:?}", bkup_list_1.len());

        for entry in &bkup_list_1 {
            if entry.is_dir() {
                path_string.clear();
                path_string.push_str(&target_base);

                let temp_string = match &entry.to_str() {
                    Some(temp_string) => temp_string,
                    None => "None value",
                };

                for x in 0..drive_count {
                    source_prefix.clear();
                    source_prefix.push_str(&temp_string[0..2].to_string());

                    if hard_drive_id[x] == source_prefix {
                        hard_drive_ct[x] += 1;
                    };
                }

                entry_length = temp_string.len();
                path_string.push_str(&temp_string[0..1].to_string());
                path_string.push_str(&temp_string[2..entry_length].to_string());
                final_path.clear();
                final_path.push(&path_string);

                if !final_path.is_dir() {
                    let _vbnm = match fs::create_dir_all(&final_path) {
                        Ok(_vbnm) => {
							my_new_dir += 1;
							info!("Created directory {:?}",&final_path);
						}
                        Err(_vbnm) => info!("{:?} {:?}", &final_path, _vbnm),
                    };
                }
            }
        }

        info!("Number of target directories created = {:?}", my_new_dir);
    }

//	The following block of code performs the actual copying of files
//	to accomplish a backup.

   
    {
        let mut entry_length: usize = 0;
        let mut final_path = PathBuf::new();
        let mut path_string = String::with_capacity(255);

        for entry in &bkup_list_1 {
            if entry.is_file() {
                path_string.push_str(&target_base);

                let temp_string = match &entry.to_str() {
                    Some(temp_string) => temp_string,
                    None => "None value",
                };

                entry_length = temp_string.len();
                path_string.push_str(&temp_string[0..1].to_string());
                path_string.push_str(&temp_string[2..entry_length].to_string());

                final_path.push(&path_string);

                if !final_path.exists() {
                    match fs::copy(&entry, &final_path) {
                        Ok(n) => {
                            bytes_copied_u64 += n;
                            files_copied_f64 += 1.0;
                            copy_new_f64 += 1.0;
                            info!("Copy new => {:?} {:?}", &entry, n);
                        }
                        Err(err) => info!("fs::copy error {:?}", err),
                    };
                } else {
                    get_meta(
                        &entry.to_path_buf(),
                        &mut source_file_attrib,
                        &mut source_creation_time,
                        &mut source_access_time,
                        &mut source_last_write_time,
                        &mut source_filesize,
                    );

                    get_meta(
                        &final_path,
                        &mut target_file_attrib,
                        &mut target_creation_time,
                        &mut target_access_time,
                        &mut target_last_write_time,
                        &mut target_filesize,
                    );

                    if source_last_write_time != target_last_write_time
                        || source_filesize != target_filesize
                    {
                        if target_file_attrib & FILE_ATTRIBUTE_READONLY == FILE_ATTRIBUTE_READONLY {
                            target_flag = true;
                            make_file_writable(&final_path, &mut target_flag);
                        }

                        if !target_flag {
                            println!("Target flag is false")
                        };

                        if target_flag {
                            match fs::copy(&entry, &final_path) {
                                Ok(n) => {
                                    bytes_copied_u64 += n;
                                    files_copied_f64 += 1.0;
                                    copy_ref_f64 += 1.0;
                                    info!("Copy ref => {:?} {:?}", &entry, n);
                                }
                                Err(err) => info!("{:?} {:?}", &entry, err),
                            };
                        }
                    }
                }
            }

            final_path.clear();
            path_string.clear();

        }
    }
	}

//	Close outer code block here


//	Output some information and then terminate the execution.

    info!("File backup operation(s) complete!");
    info!("Total files processed   = {:.0}", files_copied_f64);
    info!("Total new files copied  = {:.0}", copy_new_f64);
    info!("Total fles refreshed    = {:.0}", copy_ref_f64);
    info!("Time to perform backups = {:.2} seconds.", start_now.elapsed().as_secs_f64());

    if files_copied_f64 > 0.0 {
        info!(
            "Average duration per backup = {:.2} seconds.",
            start_now.elapsed().as_secs_f64() / files_copied_f64
        );

        let bytes_copied_f64: f64 = bytes_copied_u64 as f64;
        mean_file_size_f64 = bytes_copied_f64 / files_copied_f64;

        if bytes_copied_f64 <= KILO_BYTE {
            copy_msg_text.push_str("Bytes copied");
            display_bytes_f64 = bytes_copied_f64;
        }

        if bytes_copied_f64 > KILO_BYTE && bytes_copied_f64 <= MEGA_BYTE {
            copy_msg_text.push_str("KiloBytes copied");
            display_bytes_f64 = bytes_copied_f64 / KILO_BYTE;
        }

        if bytes_copied_f64 > MEGA_BYTE && bytes_copied_f64 <= GIGA_BYTE {
            copy_msg_text.push_str("MegaBytes copied");
            display_bytes_f64 = bytes_copied_f64 / MEGA_BYTE;
        }

        if bytes_copied_f64 > GIGA_BYTE {
            copy_msg_text.push_str("Gigabytes copied");
            display_bytes_f64 = bytes_copied_f64 / GIGA_BYTE;
        }

        info!("{:.2} {}", display_bytes_f64, copy_msg_text);
		
		copy_msg_text.clear();
		
		if mean_file_size_f64 < KILO_BYTE {
			copy_msg_text.push_str("Bytes");
			display_bytes_f64 = mean_file_size_f64;
		}
		
        if mean_file_size_f64 > KILO_BYTE && mean_file_size_f64 <= MEGA_BYTE {
            copy_msg_text.push_str("KiloBytes");
            display_bytes_f64 = mean_file_size_f64 / KILO_BYTE;
        }

        if mean_file_size_f64 > MEGA_BYTE && mean_file_size_f64 <= GIGA_BYTE {
            copy_msg_text.push_str("MegaBytes");
            display_bytes_f64 = mean_file_size_f64 / MEGA_BYTE;
        }

        if mean_file_size_f64 > GIGA_BYTE {
            copy_msg_text.push_str("Gigabytes");
            display_bytes_f64 = mean_file_size_f64 / GIGA_BYTE;
        }		
		
        info!("Average file size {:.2} {}", display_bytes_f64, copy_msg_text);
		
    }

    info!("Terminating program execution");
    std::process::exit(exitcode::OK);
}
