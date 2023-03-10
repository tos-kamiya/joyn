use std::cmp::max;
use std::fs::File;
use std::io;
use std::io::{stdout, Read, Stdout, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use clap::Parser;

mod pipe_info;
use pipe_info::get_pipe_buffer_size;

const DEFAULT_READ_BUFFER_SIZE: usize = 64 * 1024;
const NEWLINE: u8 = b'\n';

fn check_item_occurrences<T>(slice: &[T], item: &T) -> (usize, usize)
where
    T: PartialEq,
{
    let mut nl_count: usize = 0;
    let mut last_nl_pos: usize = 0;

    for (i, si) in slice.iter().enumerate() {
        if *si == *item {
            nl_count += 1;
            last_nl_pos = i;
        }
    }
    (nl_count, last_nl_pos)
}

fn line_read_and_write(
    outp: Arc<Mutex<Stdout>>,
    mut inp: File,
    read_buffer_size: usize,
) -> io::Result<usize> {
    assert!(read_buffer_size > 0);

    let mut buf = vec![b'\0'; read_buffer_size];
    let mut read_count: usize = 0;

    let mut loc: usize = 0;

    loop {
        // ensure the buffer is large enough to read another (read_buffer_size) bytes
        let bl = buf.len();
        if bl - read_count < max(bl / 2, read_buffer_size) {
            buf.resize(bl + max(bl / 2, read_buffer_size), b'\0');
        }

        // read bytes from the input into the buffer
        let c = inp.read(&mut buf[read_count..])?;
        if c == 0 { // if reached EOF
            break; // loop
        }
        read_count += c;

        // if no line can be extracted from the buffer, continue reading
        let (nl_count, last_nl_pos) = check_item_occurrences(&buf[..read_count], &NEWLINE);
        if nl_count == 0 {
            continue; // loop
        }

        // output the lines
        {
            let mut outp = outp.lock().unwrap().lock(); // take mutex of outp
            outp.write_all(&buf[..last_nl_pos + 1])?;
        }
        thread::yield_now(); // to avoid race conditions; give other threads a chance to take the mutex of outp

        // and remove the lines from the buffer
        buf.copy_within(last_nl_pos + 1.., 0);
        read_count -= last_nl_pos + 1;

        loc += nl_count;
    }

    // if the last line of the input file does not end with a newline
    if read_count > 0 {
        assert!(*buf[..read_count].last().unwrap() != NEWLINE);

        let mut outp = outp.lock().unwrap().lock();
        outp.write_all(&buf[..read_count])?; // output the line
        outp.write_all(&[NEWLINE])?; // and a newline

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

    /// Print filename, buffer size (pipe), LOC of each input file
    #[arg(long)]
    info: bool,

    /// Buffer size
    #[arg(short, long, default_value_t = DEFAULT_READ_BUFFER_SIZE)]
    buffer_size: usize,
}

fn main() -> io::Result<()> {
    let args = Cli::parse();
    if args.buffer_size == 0 {
        panic!("Error: option --buffer-size's value must be > 0");
    }

    // open input files
    let mut inps = vec![];
    for input_file in args.input.iter() {
        let f = File::open(input_file)
            .unwrap_or_else(|_err| panic!("Error: can not open file: {}", &input_file));
        inps.push(f);
    }

    // print pipe buffer size of each input file (if requested)
    if args.info {
        for (i, inp) in inps.iter().enumerate() {
            if let Some(pipe_size) = get_pipe_buffer_size(inp) {
                eprintln!(
                    "[Info] #{} {}, pipe, bufsize {}",
                    i + 1,
                    &args.input[i],
                    pipe_size
                );
            }
        }
    }

    // set up output
    let outp = Arc::new(Mutex::new(stdout()));

    // generate and run line-read-and-write threads
    let mut threads = vec![];
    for inp in inps {
        let outp = outp.clone();
        let t = thread::spawn(move || line_read_and_write(outp, inp, args.buffer_size));
        threads.push(t);
    }

    // wait until all threads terminates
    let mut locs = vec![];
    for t in threads {
        let loc = t.join().unwrap()?;
        locs.push(loc);
    }

    // print loc of each input file (if requested)
    if args.info {
        for (i, l) in locs.iter().enumerate() {
            eprintln!("[Info] #{} {}, loc {}", i + 1, &args.input[i], l);
        }
    }

    Ok(())
}
