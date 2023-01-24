use std::fs::File;
use std::io;
use std::io::{stdout, Stdout, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use clap::Parser;

const READ_BUFFER_SIZE: usize = 64 * 1024;
const NEWLINE: u8 = b'\n';

fn line_read_and_write(outp: Arc<Mutex<Stdout>>, mut inp: File) -> io::Result<usize> {
    let mut read_buf = [b'\0'; READ_BUFFER_SIZE];
    let mut write_buf = vec![];

    let mut loc: usize = 0;

    loop {
        // read some bytes from the input
        let read_count = inp.read(&mut read_buf[..])?;
        if read_count == 0 {
            break; // loop
        }

        let contains_newline = read_buf[..read_count].iter().any(|&b| b == NEWLINE);
        if ! contains_newline {
            continue; // loop
        }

        // output bytes in a line-by-line manner
        {
            let mut outp = outp.lock().unwrap().lock(); // here, take mutex of outp
            for &b in &read_buf[..read_count] {
                write_buf.push(b);
                if b == NEWLINE {
                    loc += 1;
                    outp.write_all(&write_buf)?;
                    write_buf.clear();
                }
            }
        }
        thread::yield_now(); // to avoid race conditions; give other threads a chance to take the mutex of outp
    }

    // when the last line does not terminated with a line number
    if ! write_buf.is_empty() {
        assert!(*write_buf.last().unwrap() != NEWLINE);

        loc += 1;

        // add a new-line char if the last line does not have it
        write_buf.push(NEWLINE);

        // then output the line
        let mut outp = outp.lock().unwrap().lock();
        outp.write_all(&write_buf)?;
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

    // print summary (if requested)
    if args.summary {
        for (i, l) in locs.iter().enumerate() {
            eprintln!("[Info] {} lines read from input #{}: {}", l, i + 1, &args.input[i]);
        }
    }

    Ok(())
}
