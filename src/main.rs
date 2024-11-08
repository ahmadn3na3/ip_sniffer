use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc::{channel, Sender};
use std::{env, net::IpAddr, process, str::FromStr};
use std::{io, thread};

// Max number of ports that can be scanned

const MAX: u16 = 65535;

struct Arguments {
    threads: u16,
    ipaddr: IpAddr,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        } else if args.len() > 4 {
            return Err("too many arguments");
        }
        let f = args[1].clone();
        if let Ok(ipaddr) = IpAddr::from_str(&f) {
            return Ok(Arguments {
                threads: 4,
                ipaddr,
            });
        } else {
            let f = args[1].clone();
            if (f.contains("-h") || f.contains("-help") && f.len() == 2) {
                println!(
                    "Usage: -j to select how many threads you want \r\n
                      -h or -help to show this help message"
                );
                return Err("help");
            } else if f.contains("-h") || f.contains("-help") {
                return Err("too many arguments");
            } else if f.contains("-j") {
                let ipaddr = match IpAddr::from_str(&args[3]) {
                    Ok(s) => s,
                    Err(_) => return Err("not a valid IPADDR; must be IPv4 or IPv6"),
                };
                let threads = match args[2].parse::<u16>() {
                    Ok(i) => i,
                    Err(_) => return Err("failed to parse thread number"),
                };
                return Ok(Arguments {
                    ipaddr: ipaddr,
                    threads,
                });
            } else {
                return Err("invalid syntax");
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let parsed_args = Arguments::new(&args).unwrap_or_else({
        |err| {
            if err.contains("help") {
                process::exit(0);
            } else {
                eprintln!("{} problem parsing arguments: {}", program, err);
                process::exit(0);
            }
        }
    });
    let num_threads = parsed_args.threads;
    let addr = parsed_args.ipaddr;
    let (tx, rx) = channel();
    for i in 0..num_threads {
        let tx = tx.clone();
        thread::spawn(move || scan(tx, i, addr, num_threads));
    }
    let mut out = vec![];
    drop(tx);
    for v in rx {
        out.push(v);
    }
    println!("");
    out.sort();
    for v in out {
        println!("{} is open", v);
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port: u16 = start_port + 1;
    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => {}
        }
        if (MAX - port) <= num_threads {
            break;
        }
        port += num_threads;
    }
}
