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

fn find_positions_of<T>(slice: &[T], item: &T) -> Vec<usize>
where
    T: PartialEq,
{
    let mut poss = vec![];
    for (i, si) in slice.iter().enumerate() {
        if *si == *item {
            poss.push(i);
        }
    }
    poss
}

fn line_read_and_write(
    outp: Arc<Mutex<Stdout>>,
    mut inp: File,
    read_buffer_size: usize,
) -> io::Result<usize> {
    assert!(read_buffer_size > 0);

    let mut read_buf = vec![b'\0'; read_buffer_size];
    let mut write_buf = vec![];

    let mut loc: usize = 0;

    loop {
        // read some bytes from the input
        let read_count = inp.read(&mut read_buf[..])?;
        if read_count == 0 {
            break; // loop
        }
        let read_buf = &read_buf[..read_count];

        // if the read bytes does not contain new-line chars, just add the bytes to write buffer
        let nl_poss = find_positions_of(read_buf, &NEWLINE);
        if nl_poss.is_empty() {
            write_buf.extend_from_slice(read_buf);
            continue; // loop
        }

        // otherwise,

        // extract lines and add them to write buffer,
        let the_last_nl_pos = nl_poss.last().unwrap();
        let (lines, remains) = read_buf.split_at(the_last_nl_pos + 1);
        write_buf.extend_from_slice(lines);

        // output write buffer contents,
        {
            let mut outp = outp.lock().unwrap().lock(); // take mutex of outp
            outp.write_all(&write_buf)?;
        }
        thread::yield_now(); // to avoid race conditions; give other threads a chance to take the mutex of outp

        // and make write buffer contains the remaining bytes
        write_buf.clear();
        write_buf.extend_from_slice(remains);

        loc += nl_poss.len();
    }

    if !write_buf.is_empty() {
        // if the last line is not terminated by a new-line char
        assert!(*write_buf.last().unwrap() != NEWLINE);

        // add a new-line char
        write_buf.push(NEWLINE);

        // then output the line
        let mut outp = outp.lock().unwrap().lock();
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

    /// Print filename, buffer size (pipe), LOC of each input file
    #[arg(long)]
    info: bool,

    /// Buffer size
    #[arg(short, long, default_value_t = DEFAULT_READ_BUFFER_SIZE)]
    buffer_size: usize,
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    // open input files
    let mut inps = vec![];
    for input_file in args.input.iter() {
        let f = File::open(input_file)
            .unwrap_or_else(|_err| panic!("Error: can not open file: {}", &input_file));
        inps.push(f);
    }

    // print pipe buffer size (if requested)
    if args.info {
        for (i, inp) in inps.iter().enumerate() {
            if let Some(pipe_size) = get_pipe_buffer_size(inp) {
                eprintln!("[Info] #{} {}, pipe, bufsize {}", i + 1, &args.input[i], pipe_size);
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

    // print loc (if requested)
    if args.info {
        for (i, l) in locs.iter().enumerate() {
            eprintln!(
                "[Info] #{} {}, loc {}",
                i + 1,
                &args.input[i],
                l
            );
        }
    }

    Ok(())
}
