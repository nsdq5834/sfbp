//  Program: sfbp
//  Author: Bill Meany
//  Date: 04/03/2020
//  Version: 1.0.0
//  Revision date: 12/31/2020
//  Revision: 1.0.0

//	Simple File Backup Program
//	Platform: Windows

//	Bring in code we need.

use log::info;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process;
use walkdir::WalkDir;

//	Get some local functions from lib.rs

use sfbp::house_keeping;
use sfbp::construct_lf_name;
use sfbp::setup_logger;
use sfbp::get_meta;
use sfbp::make_file_writable;

// Define some constants

const KILO_BYTE: f64 = 1024.0;
const MEGA_BYTE: f64 = KILO_BYTE * KILO_BYTE;
const GIGA_BYTE: f64 = MEGA_BYTE * KILO_BYTE;
const NUMB_PARM: u16 = 2;
const DEBUG_FLAG: bool = false;
const RC00: i32 = 0;

//	Windows file system constants. We only make use of two of them
//	but have them all listed for documentation purposes.

const FILE_ATTRIBUTE_READONLY: u32 =            0x00000001;
const FILE_ATTRIBUTE_HIDDEN: u32 =              0x00000002;
const FILE_ATTRIBUTE_SYSTEM: u32 =              0x00000004;
const FILE_ATTRIBUTE_DIRECTORY: u32 =           0x00000010;
const FILE_ATTRIBUTE_ARCHIVE: u32 =             0x00000020;
const FILE_ATTRIBUTE_DEVICE: u32 =              0x00000040;
const FILE_ATTRIBUTE_NORMAL: u32 =              0x00000080;
const FILE_ATTRIBUTE_TEMPORARY: u32 =           0x00000100;
const FILE_ATTRIBUTE_SPARSE_FILE: u32 =         0x00000200;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 =       0x00000400;
const FILE_ATTRIBUTE_COMPRESSED: u32 =          0x00000800;
const FILE_ATTRIBUTE_OFFLINE: u32 =             0x00001000;
const FILE_ATTRIBUTE_NOT_CONTENT_INDEXED: u32 = 0x00002000;
const FILE_ATTRIBUTE_ENCRYPTED: u32 =           0x00004000;
const FILE_ATTRIBUTE_INTEGRITY_STREAM: u32 =    0x00008000;
const FILE_ATTRIBUTE_VIRTUAL: u32 =             0x00010000;
const FILE_ATTRIBUTE_NO_SCRUB_DATA: u32 =       0x00020000;
const FILE_ATTRIBUTE_EA: u32 =                  0x00040000;

//	Executable code starts here.

fn main() {
	info!("Beginning program execution");
	let mut log_file_name = String::with_capacity(255);
	let log_file_prefix = String::from("\\Log_");
	let mut prog_name = String::with_capacity(25);
	let mut _bkup_source = String::new();
	let mut _exclude_source = String::new();
	let mut _target_base = String::new();
	let mut _copy_message = String::new();
	
	let mut bytes_copied_u64: u64 = 0;
	let bytes_copied_f64: f64 = 0.0;
	let mut display_bytes_f64: f64 = 0.0;
	let mut files_copied_f64: f64 = 0.0;
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
	
	let mut _bkup_s1 = Vec::<String>::new();
	let mut _bkup_s2 = Vec::<PathBuf>::new();
	let mut _excl_s1 = Vec::<PathBuf>::new();

//	Do some simple housekeeping using house_keeping from lib.rs

	house_keeping(NUMB_PARM,&mut prog_name);

//	Build the log file name using construct_lf_name from lib.rs

	construct_lf_name(&mut log_file_name,&log_file_prefix,&prog_name);

//	Create the log file using setup_logger from lib.rs

	setup_logger(&log_file_name,DEBUG_FLAG)
		.expect("failed to initialize logging.");

//	Log file has been opened so we can proceed.

	info!("Beginning program execution");
	
//	Construct the name of our parameter file, and then create a path to it.
//	Create a code block so that objects, variable, etc. related to the 
//	parameter file go away when we are done with them.

{

	let mut parm_file = prog_name.clone();
	parm_file.push_str(".parms");

	info!("Attempting to open {}",parm_file);

//	let fh = File::open(&parm_file);
	
	let fh = match File::open(&parm_file) {
		Ok(file) => file,
		Err(err) => {
			info!("{}", err);
			info!("Terminating program execution");
			process::exit(16);
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
			let bkup_parms:Vec<&str>= line.split("=").collect();			
			if bkup_parms[0].trim() == "BackupSource" {
				_bkup_source = bkup_parms[1].trim().to_string();
			}
			if bkup_parms[0].trim() == "ExcludeSource" {
				_exclude_source = bkup_parms[1].trim().to_string();
			}
			if bkup_parms[0].trim() == "BackupBaseLocation" {
				_target_base = bkup_parms[1].trim().to_string();
			}
		}
    }

	info!("Source directory list is {}", _bkup_source);
	info!("Exclude directory list is {}", _exclude_source);
	info!("Target backup location is {}", _target_base);
	
	if _bkup_source == "" {
		info!("No source directory list provided");
		process::exit(RC00);
	}
	
	if _exclude_source == "" {
		info!("No exclude directory list provided");
		process::exit(RC00);
	}
	
	if _target_base == "" {
		info!("No target directory base provided");
		process::exit(RC00);
	}
	
}

//	This code block processes the source directories file.
//	We will build the list of directories into _bkup_s1.

{

	let fh = match File::open(_bkup_source) {
		Ok(file) => file,
		Err(err) => {
			info!("{}", err);
			info!("Terminating program execution");
			process::exit(RC00);
		}
	};

	let pf_handle = BufReader::new(fh);
	
	    for line in pf_handle.lines() {
        let line = line.expect("Unable to read line");
		_bkup_s1.push(line);
    }
	
		let _num_bkup_s1 = _bkup_s1.len();
		info!("Number of base directories to backup is {}",_num_bkup_s1);

}

//	This code block processes the exclude directories file.
//	We will build the list of exclude directories into _excl_s1.

{

	let fh = match File::open(_exclude_source) {
		Ok(file) => file,
		Err(err) => {
			info!("{}", err);
			info!("Terminating program execution");
			process::exit(RC00);
		}
	};

	let pf_handle = BufReader::new(fh);
	
	    for line in pf_handle.lines() {
        let line = line.expect("Unable to read line");
		_excl_s1.push(PathBuf::from(&line));
    }
	
		let _num_excl_s1 = _excl_s1.len();
		info!("Number of directories to exclude is {}",_num_excl_s1);

}

{

//	Following code block obtains the metadata about the provided target
//	backup directory and validates that it is a directory.

	let mut _work_path_buf = PathBuf::new();
	_work_path_buf.push(&_target_base);
	
	get_meta(&_work_path_buf,
			&mut source_file_attrib,
			&mut source_creation_time,
			&mut source_access_time,
			&mut source_last_write_time,
			&mut source_filesize);
			
	if source_file_attrib & FILE_ATTRIBUTE_DIRECTORY ==
		FILE_ATTRIBUTE_DIRECTORY {
		info!("{} validated as a directory structure", _target_base);
	}
	else {
		info!("{} is not a valid directory structure!", _target_base);
		info!("Terminating program execution");
		process::exit(RC00);
	}
	
}

//
//	Next step is to build a list of all the files and directories that may
//	be candidates for a backup.
//
//	_bkup_s1 contains the preliminary list of source directories.
//	

{
	
	let mut my_count: i32 = 0;
	
	for current_source in _bkup_s1 {
		for entry in WalkDir::new(&current_source).min_depth(0).
			sort_by(|a,b| a.file_name().cmp(b.file_name())) {
				
				match entry {
					Ok(entry) => {
						my_count = my_count + 1;
						_bkup_s2.push(entry.path().to_path_buf());
					} ,
					Err(entry) => {
						info!("Error obtaining directory entry {:?}", entry);
					}
				};
		}
	}
	
	info!("Number of potential backups = {:?}", _bkup_s2.len());
}


//	The following code block processes the entries in the _bkup_s2 vector.
//	These are all of the entries that were discovered in the previous block
//	and are either a directory or a file entry. The purpose of this block
//	is to take each entry that is a source directory and determine if the
//	associated target directory exists. To do this, we take a path entry in
//	_bkup_s2 and test to see if it is a directory. If it is, then we will 
//	make a string copy of the path, strip out the colon and then prefix
//	the result with _target_base to create the target path. We test to see if
//	the target exists, and if it does not we will create it.
//
//	There are still some hardcoded constants we need to remove.

{
	
	let mut my_count_c: i32 = 0;
	let mut my_count_d: i32 = 0;
	let mut my_new_dir: i32 = 0;
	let mut entry_length: usize = 0;
	
	let mut final_path = PathBuf::new();
	let mut path_string = String::with_capacity(100);
	
	_bkup_s2.sort();
		
	for entry in &_bkup_s2 {

		if entry.is_dir() {			
				
			if entry.starts_with("C:") {
				my_count_c += 1;
			}
			if entry.starts_with("D:") {
				my_count_d += 1;
			}

			path_string.clear();
			path_string.push_str(&_target_base);
			
			let temp_string = match &entry.to_str() {
				Some(temp_string) => temp_string,
				None => "None value",
			};
			
			entry_length = temp_string.len();
			path_string.push_str(&temp_string[0..1].to_string());
			path_string.push_str(&temp_string[2..entry_length].to_string());
			final_path.clear();
			final_path.push(&path_string);
			
			if !final_path.is_dir() {
				let _vbnm = match fs::create_dir_all(&final_path) {
					Ok(_vbnm) => my_new_dir += 1,
					Err(_vbnm) => println!("{:?} {:?}", &final_path, _vbnm)
				};
	
			
			}
		}
	}

	info!("Number of source directories on C: = {:?}", my_count_c);
	info!("Number of source directories on D: = {:?}", my_count_d);
	info!("Number of target directories created = {:?}", my_new_dir);
	
}

//	The following block of code performs the actual copying of files
//	to accomplish a backup.

{

	let mut entry_length: usize = 0;	
	let mut final_path = PathBuf::new();	
	let mut path_string = String::with_capacity(100);
	
	for entry in &_bkup_s2 {
		
		if entry.is_file() {
			
			path_string.push_str(&_target_base);
			
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
						info!("Copied => {:?} {:?}", &entry, n);
					},
					Err(err) => info!("fs::copy error {:?}", err),
				};
			}
			else {
				get_meta(&entry.to_path_buf(),
			             &mut source_file_attrib,
			             &mut source_creation_time,
			             &mut source_access_time,
			             &mut source_last_write_time,
			             &mut source_filesize);
						 
				get_meta(&final_path,
			             &mut target_file_attrib,
			             &mut target_creation_time,
			             &mut target_access_time,
			             &mut target_last_write_time,
			             &mut target_filesize);
				}
			if source_last_write_time != target_last_write_time ||
				source_filesize != target_filesize {	
  
				   	if target_file_attrib & FILE_ATTRIBUTE_READONLY ==
						FILE_ATTRIBUTE_READONLY {
							target_flag = true;
							make_file_writable(&final_path, &mut target_flag);
						}
						
					if !target_flag { println!("Target flag is false") };
					
					if target_flag {
						match fs::copy(&entry, &final_path) {
							Ok(n) => {
								bytes_copied_u64 += n;
								files_copied_f64 += 1.0;
								info!("Copied => {:?} {:?}", &entry, n);
							},
							Err(err) => info!("{:?} {:?}", &entry, err),					   
						};
					}			   
			   
			}
			
		}
		
		final_path.clear();
		path_string.clear();
	}

}

	info!("Total files copied = {:.0}", files_copied_f64);
	
	if files_copied_f64 > 0.0 {
	
		let bytes_copied_f64: f64 = bytes_copied_u64 as f64;
		mean_file_size_f64 = bytes_copied_f64 / files_copied_f64;
	
		if bytes_copied_f64 < KILO_BYTE {
			_copy_message.push_str("Bytes copied");
			display_bytes_f64 = bytes_copied_f64;
		}

		if bytes_copied_f64 > KILO_BYTE && bytes_copied_f64 < MEGA_BYTE {
			_copy_message.push_str("KiloBytes copied");
			display_bytes_f64 = bytes_copied_f64 / KILO_BYTE;
		}
	
		if bytes_copied_f64 > MEGA_BYTE && bytes_copied_f64 < GIGA_BYTE {
			_copy_message.push_str("MegaBytes copied");
			display_bytes_f64 = bytes_copied_f64 / MEGA_BYTE;
		}
		else {
			_copy_message.push_str("Gigabytes copied");
			display_bytes_f64 = bytes_copied_f64 / MEGA_BYTE;
		}
	
		info!("{:.2} {}", display_bytes_f64, _copy_message);
		info!("Average file size {:.0} bytes", mean_file_size_f64);
	
	}
	
	info!("Terminating program execution");

}