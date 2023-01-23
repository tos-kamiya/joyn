use std::fs::File;
use std::io;
use std::io::{stdout, Stdout, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use clap::Parser;

const READ_BUFFER_SIZE: usize = 64 * 1024;
const EOL: u8 = b'\n';

fn line_read_and_write(outp: Arc<Mutex<Stdout>>, mut inp: File) -> io::Result<usize> {
    let mut read_buf = [b'\0'; READ_BUFFER_SIZE];
    let mut write_buf = vec![];

    let mut loc: usize = 0;

    loop {
        let read_count = inp.read(&mut read_buf[..])?;
        if read_count == 0 {
            break; // loop
        }
        for b in &read_buf[..read_count] {
            let b = *b;
            write_buf.push(b);
            if b == EOL {
                loc += 1;
                let mut outp = outp.lock().unwrap();
                outp.write_all(&write_buf)?;
                write_buf.clear();
            }
        }
    }

    if ! write_buf.is_empty() {
        // if the file is not terminated by a new-line char, add it.
        let b = write_buf.last().unwrap();
        if ! (*b == EOL) {
            write_buf.push(EOL);
        }

        let mut outp = outp.lock().unwrap();
        outp.write_all(&write_buf)?;
        loc += 1;
    }

    Ok(loc)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// Join input files. Create a thread for each input file that reads a line, and write a line each time any thread reads it.
struct Cli {
    /// Input files
    input: Vec<String>,

    /// Print LOC of each input file on exit
    #[arg(short, long)]
    summary: bool,
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    // open input files
    let mut inps = vec![];
    for input_file in args.input.iter() {
        let f = File::open(input_file).unwrap_or_else(|_err|
           panic!("Error: can not open file: {}", &input_file));
        inps.push(f);
    }

    // set up output
    let outp = Arc::new(Mutex::new(stdout()));

    // generate and run line-read-and-write threads
    let mut threads = vec![];
    for inp in inps {
        let outp = outp.clone();
        let t = thread::spawn(move || line_read_and_write(outp, inp));
        threads.push(t);
    }

    // wait until all threads terminates
    let mut locs = vec![];
    for t in threads {
        let loc = t.join().unwrap()?;
        locs.push(loc);
    }

    // Print summary (if requested)
    if args.summary {
        for (i, l) in locs.iter().enumerate() {
            eprintln!("[Info] {} lines read from input #{}: {}", l, i + 1, &args.input[i]);
        }
    }

    Ok(())
}
