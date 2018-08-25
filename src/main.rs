pub mod torrent;

use std::net::{ToSocketAddrs};
use std::default::Default;
use std::io::{self, Write};
use std::fs::{File};
use std::io::prelude::*;

extern crate bip_bencode;
extern crate hyper;

use bip_bencode::{BencodeRef, BDecodeOpt};
use hyper::Client;
use hyper::rt::{self, Future, Stream};


use torrent::torrent::{Torrent};
// 
// fn extract_pieces()
/*
    TODO:
        1. move torrent file opening parts to a separate file
        2. apply command line option for path to torrent file
        3. expand on torrent struct
        4. ???
        5. Profit.
*/ 
fn main() {
    let mut torrent_file: File = File::open("./steven_universe.torrent").unwrap();
    let mut contents = Vec::new();
    let udp = "9.rarbg.me:2710".to_socket_addrs().unwrap();
    println!("{:?}", udp);
    torrent_file.read_to_end(&mut contents);
    // println!("CONTENTS: {:?}", contents);
    let torrent_content = BencodeRef::decode(&contents, BDecodeOpt::default()).unwrap();
    let torrent = Torrent::new(&torrent_content);

    let uri = "http://httpbin.org/ip".parse().unwrap();
    rt::run(fetch_url(uri));
}

fn fetch_url(url: hyper::Uri) -> impl Future<Item=(), Error=()> {
    let client = Client::new();
    
    client
        .get(url)
        .and_then(|res| {
            println!("Response: {}", res.status());
            res.into_body().for_each(|chunk| {
                    io::stdout()
                        .write_all(&chunk)
                        .map_err(|e| {
                            panic!("example expects stdout is open, error={}", e)
                        })
                })
        })
        .map_err(|err| {
            println!("Error: {}", err);
        })
}
