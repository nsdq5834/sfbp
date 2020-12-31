///  Program: lib.rs
///  Author: Bill Meany
///  Date: 04/24/2020
///  Version: 1.0.0
///  Revision date: 12/28/2020
///  Revision: 1.0.8

use std::convert::TryInto;
use std::env;
use std::fs;
use log::info;
use std::os::windows::prelude::*;
use std::path::PathBuf;
use std::process;
//use chrono::prelude::*;

//	Simple housekeeping routine. Check to see if the correct number of para-
//	meters were present on the command line. If so, isolate the program name
//	and return it to the caller.
//
//	Function parameters:
//
//	nparm - number of command line arguments expected
//	pgm_name - name of the program we were invoked under
//

pub fn house_keeping(nparm: u16,pgm_name: &mut String) {

	let cli_args: Vec<String> = env::args().collect();

	if cli_args.len() != nparm.try_into().unwrap() {
		println!("Incorrect number of parameters provided");
		process::exit(0)
	}
	
	let cli_arg01 = &cli_args[0];
	let cli_arg01_tokens: Vec<&str>= cli_arg01.split("\\").collect();
	let full_prog_name: Vec<&str> = cli_arg01_tokens[2].split(".").collect();
	let just_program: String = full_prog_name[0].to_owned();
	pgm_name.push_str(&just_program);
	
}

//	Build a name for our log file.
//	
//	Function parameters:
//
//	lfn - mutable reference to callers variable
//	lfp - reference to the log file prefix value
//	jpn - reference to the current program name
//
//	Example log file name might be:
//
//	C:\Logs\my_program\Log_20200101-070707.txt
//

pub fn construct_lf_name(lfn: &mut String, lfp: &String, jpn: &String) {

	let cli_args: Vec<String> = env::args().collect();
	let cli_arg02 = &cli_args[1];
	lfn.push_str(&cli_arg02.to_owned());
	lfn.push_str(&jpn);
	lfn.push_str(lfp);
	
	let right_now: String = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
	lfn.push_str(&right_now);
	lfn.push_str(".txt");
	
}

//	The following simple function is used to build a new logging instance. We
//	are making use of the functionality provided by the fern crate. See the
//	document provided in that crate at crate.io.
//
//
//	Function parameters:
//
//	lfn - the name of the log file we want to create and write to.

pub fn setup_logger(lfn: &str, stdflg: bool) -> Result<(), fern::InitError> {

    let base_config = fern::Dispatch::new();
	
	let file_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} {} {} {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(&lfn)?);

	let stdo_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} {} {} {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout());

	if stdflg {
	base_config
		.chain(file_config)
		.chain(stdo_config)
		.apply()?;
	}
	else {
	base_config
		.chain(file_config)
		.apply()?;
	}	
    Ok(())
	
}


//	Simple function to obtain file metadata. This implementation is
//	specific to the Windows environment.
//	We use the match construct so we can gracefully handle any error(s)
//	that might occur.

pub fn get_meta(file_entry: &PathBuf,
				my_file_attrib: &mut u32,
				my_creation_time: &mut u64,
				my_access_time: &mut u64,
				my_last_write_time: &mut u64,
				my_filesize: &mut u64) {
	
    let _metadata = match fs::metadata(file_entry) {
		Ok(_metadata) => {
			*my_file_attrib = _metadata.file_attributes();
			*my_creation_time = _metadata.creation_time();
			*my_access_time = _metadata.last_access_time();
			*my_last_write_time = _metadata.last_write_time();
			*my_filesize = _metadata.file_size()
			} ,
		Err(_metadata) => {
			*my_creation_time = 0;
			}
	} ;
}

//	Simple function to turn off the readonly setting on a file.
//	Match construct is used to handle errors.

pub fn make_file_writable(file_entry: &PathBuf,
						  file_flag: &mut bool) {
						  
	let _metadata = match fs::metadata(file_entry) {
	
		Ok(_metadata) => {
			let mut _my_perms = _metadata.permissions();
			_my_perms.set_readonly(false);
			
			let _my_result = match fs::set_permissions(file_entry, _my_perms) {
				Ok(_my_result) => _my_result,
				Err(_my_result) => {
					info!("fs::set_permissions error = {:?}", _my_result);
					*file_flag = false;
					}
			};
		},
		Err(_metadata) => {
			info!("Unable to obtain metadata for {:?}", file_entry);
			*file_flag = false;
		}
	};
}
