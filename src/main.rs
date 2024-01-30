use std::io::{Write, Read};

use clap::Parser;
use gkd_protocol::{ControlCodes, ResultCodes};

mod gkd_protocol;

const CONFIG_FILE: &str = "/etc/gkd-unlock/config.yaml";

#[derive(Parser, Debug)]
struct Args {
    /// Path to the config file
    /// 
    /// Contains the following fields (yaml):
    /// ```yaml
    /// config:
    ///    password_file_path: "{{PASSWORD_FILE_ABSOLUTE_PATH}}"
    /// ```
    #[arg(short, long, default_value = CONFIG_FILE)]
    config_path: String,
}

#[derive(serde::Deserialize, Debug)]
struct Config {
    password_file_path: String,
}

fn main() {
    let args = Args::parse();

    // Parse the config file
    let passwd = parse_config(args);

    // Communicate with gnome-keyring-daemon
    authenticate_gkd(passwd)
        .expect("Could not authenticate with gnome-keyring-daemon");
}


/// Read config file from `/etc/gkd-unlock.conf` 
/// or from the path provided via -c flag.
fn parse_config(args: Args) -> String {
    let config_path = args.config_path;
    let config_file = std::fs::read_to_string(config_path)
        .expect("Unable to read config file");

    // Parse the config file
    let config: Config = serde_yaml::from_str(&config_file)
        .expect("Unable to parse config file");

    let pass_file = std::fs::read_to_string(config.password_file_path);
    if pass_file.is_err() {
        eprintln!("File could not be opened, probably the file is not present.");
        std::process::exit(0);
    }

    // Get password and go ahead
    pass_file.unwrap()
}

/// Authenticate with gnome-keyring-daemon
fn authenticate_gkd(passwd: String) -> Result<(), Box<dyn std::error::Error>> {
    
    // Check for GNOME_KEYRING_CONTROL env variable -> use it if present
    let gkd_control = std::env::var("GNOME_KEYRING_CONTROL");
    // Check for XDG_RUNTIME_DIR env variable -> use it as fallback if present
    let xdg_runtime_dir = std::env::var("XDG_RUNTIME_DIR");

    let mut socket: std::os::unix::net::UnixStream;
    
    if let Ok(gkd_control) = gkd_control {
        // Append /control to the path to get the socket path
        let gkd_control = format!("{}/control", gkd_control);

        // Create a unix domain socket
        socket = std::os::unix::net::UnixStream::connect(gkd_control)?
    } else if let Ok(xdg_runtime_dir) = xdg_runtime_dir {
        // Append /keyring/control to the path to get the socket path
        let xdg_runtime_dir = format!("{}/keyring/control", xdg_runtime_dir);

        // Create a unix domain socket
        socket = std::os::unix::net::UnixStream::connect(xdg_runtime_dir)?
    } else {
        // If both env variables are not present, then exit
        eprintln!("Unable to find gnome-keyring-daemon socket.");
        std::process::exit(0);
    }

    // === Unlock the keyring ===

    // Write credentials byte which must be 0x00
    let init_code = 0x00_u8;
    if !socket.write(&init_code.to_be_bytes()).is_ok_and(|bytes| bytes == 1) {
        return Err("Could not write init code to socket".into());
    }

    // Write the OP code length that follows 
    // (4B pack size; 4B opcode; 4B passwd length; passwd)
    let op_len: u32 = (8 + 4 + passwd.len()) as u32;

    // 1. Write packet size
    if !socket.write(&op_len.to_be_bytes()).is_ok_and(|bytes| bytes == 4) {
        return Err("Could not write packet size to socket".into());
    }

    // 2. Write the unlock command (0x01)
    let unlock_code = ControlCodes::Unlock as u32;
    if !socket.write(&unlock_code.to_be_bytes()).is_ok_and(|bytes| bytes == 4) {
        return Err("Could not write unlock code to socket".into());
    }

    // 3. Write the password length
    let passwd_len = passwd.len() as u32;
    let passwd_bytes = &passwd_len.to_be_bytes();
    if !socket.write(passwd_bytes).is_ok_and(|bytes| bytes == 4) {
        return Err("Could not write password length to socket".into());
    }

    // 4. Write the password (utf8)
    let pass_utf8 = passwd.as_bytes();
    if !socket.write(pass_utf8).is_ok_and(|bytes| bytes == passwd.len()) {
        return Err("Could not write password to socket".into());
    }

    // Get the Result

    // 1. Read the result length
    let mut result_length = [0u8; 4];
    socket.read_exact(&mut result_length)?;
    
    // It must be 8 bytes because first 4 bytes are 0 
    // and next 4 bytes are the result code
    if u32::from_be_bytes(result_length) != 8 {
        return Err("Result length is not 8 bytes".into());
    }
    
    // 2. Read the result code (rest of the 8 bytes => 4 bytes)
    let mut result_code_rsp = [0u8; 4];
    socket.read_exact(&mut result_code_rsp)?;

    // Convert the result code to u32 and then to the enum
    let result_code: ResultCodes = u32::from_be_bytes(result_code_rsp).into();

    match result_code {
        ResultCodes::Ok => {
            // Successfully authenticated with gnome-keyring-daemon
            return Ok(())
        },
        ResultCodes::Denied => {
            return Err("Authentication denied by gnome-keyring-daemon".into());
        },
        ResultCodes::Failed => {
            return Err("Authentication failed".into())
        },
        ResultCodes::NoDaemon => {
            return Err("gnome-keyring-daemon is not running".into())
        },
    }
}