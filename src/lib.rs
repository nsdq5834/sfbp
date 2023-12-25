///  Program: lib.rs
///  Author: Bill Meany
///  Date: 04/24/2020
///  Version: 1.0.0
///  Revision date: 11/21/2021
///  Revision: 1.0.10

use log::info;
use std::fs;
use std::os::windows::prelude::*;
use std::path::Path;

pub fn construct_lf_name(lfn: &mut String, lfp: &str, jpn: &str) {
//    let cli_args: Vec<String> = env::args().collect();
//    let cli_arg02 = &cli_args[1];
//    lfn.push_str(&cli_arg02.to_owned());
//	println!("{:?}",&lfn);
    lfn.push_str(lfp);
    lfn.push_str(jpn);
    lfn.push_str("\\Log_");

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
        base_config.chain(file_config).chain(stdo_config).apply()?;
    } else {
        base_config.chain(file_config).apply()?;
    }
    Ok(())
}

//	Simple function to obtain file metadata. This implementation is
//	specific to the Windows environment.
//	We use the match construct so we can gracefully handle any error(s)
//	that might occur. If we cannot obtain the metadata we will set the
//	file creation time to zero.

//	Function parameters:
//	file_entry reference to a path buffer for the file we want to work on.
//	my_file_attribute file attribute settings.
//	my_creation_fime date and time when the file was created.
//	my_access_time date and time when file was last accessed.
//	my_last_write date and time when the file was written to.
//	my_filesize size of the file in bytes.

pub fn get_meta(
    file_entry: &Path,
    my_file_attrib: &mut u32,
    my_creation_time: &mut u64,
    my_access_time: &mut u64,
    my_last_write_time: &mut u64,
    my_filesize: &mut u64,
) {
    let _metadata = match fs::metadata(file_entry) {
        Ok(_metadata) => {
            *my_file_attrib = _metadata.file_attributes();
            *my_creation_time = _metadata.creation_time();
            *my_access_time = _metadata.last_access_time();
            *my_last_write_time = _metadata.last_write_time();
            *my_filesize = _metadata.file_size()
        }
        Err(_metadata) => {
            *my_creation_time = 0;
        }
    };
}

//	Simple function to turn off the readonly setting on a file.
//	Match construct is used to handle errors.
//
//	Function parameters:
//	file_entry reference to a path buffer for the file we want to work on.
//	file_flag boolean used to indicate success or failure.

pub fn make_file_writable(file_entry: &Path, file_flag: &mut bool) {
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
        }
        Err(_metadata) => {
            info!("Unable to obtain metadata for {:?}", file_entry);
            *file_flag = false;
        }
    };
}
