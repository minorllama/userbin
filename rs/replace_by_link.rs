use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Error, ErrorKind};
use std::path::Path;
use std::process::{self, Command, Stdio};
use std::thread::{self, JoinHandle};

fn stream_pipe<R: Read + Send + 'static>(pipe: R, prefix: &'static str, log_to_stderr: bool) -> JoinHandle<()> {
    thread::spawn(move || {
        let reader = BufReader::new(pipe);
        if log_to_stderr {
            for line in reader.lines().flatten() {
                eprintln!("[{}] {}", prefix, line);
            } 
        } else {
            for line in reader.lines().flatten() {
                println!("[{}] {}", prefix, line);
            } 
        }
    })
}

fn shell(cmd: &mut Command) -> std::io::Result<()> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn()?;
    // stream pipes in seperate threads
    let stdout = child.stdout.take().expect("failed to open stdout");
    let stderr = child.stderr.take().expect("failed to open stderr");

    let stdout_handle = stream_pipe(stdout, "cp stdout", false);
    let stderr_handle = stream_pipe(stderr, "cp stderr", true);

    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("'cp' failed: {}", status),
        ))
    }
}

fn move_path(src: &Path, dst: &Path) -> std::io::Result<()> {
    // try rename first, will fail on trying to cross filesystems
    if fs::rename(src, dst).is_ok() {
        return Ok(());
    } else {
        let mut cp = Command::new("cp");
        let cp_args = cp.arg("-Rav") // (R)ecursive + (a)rchive + (v)erbose 
          .arg(src)
          .arg(dst);
        shell(cp_args)?;
        // clean up for move
        if src.is_dir() {
            fs::remove_dir_all(src)?;
        } else {
            fs::remove_file(src)?;
        }
        Ok(())
    }
}


fn move_and_symlink(target: &str, dest_dir: &str) -> io::Result<()> {
    let target_path = fs::canonicalize(target).map_err(|_| {
        Error::new(
            ErrorKind::NotFound,
            format!("target '{target}' does not exist"),
        )
    })?;

    // create dest_dir if missing
    if !fs::metadata(dest_dir).map(|m| m.is_dir()).unwrap_or(false) {
        fs::create_dir_all(dest_dir)?;
        println!("    destination '{dest_dir}' created.");
    }

    let dest_dir_path = fs::canonicalize(dest_dir)?;

    if !dest_dir_path.is_dir() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("destination '{dest_dir_path:?}' is not a valid directory"),
        ));
    }

    let file_name = target_path.file_name().ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("invalid target path name: '{target_path:?}'"),
        )
    })?;

    let new_location = dest_dir_path.join(file_name);

    if new_location.exists() {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("'{new_location:?}' already exists in destination"),
        ));
    }

    move_path(&target_path, &new_location)?;
    println!("    moved: '{target_path:?}' -> '{new_location:?}'");

    std::os::unix::fs::symlink(&new_location, &target_path)?;
    println!("    linked: '{target_path:?}' -> '{new_location:?}'");

    Ok(())
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("\n\t move_and_symlink $target $dest_dir");
        process::exit(1);
    }

    if let Err(e) = move_and_symlink(&args[1], &args[2]) {
        eprintln!("{}", e);
        process::exit(1);
    } else {
        process::exit(0);   
    }
    
}