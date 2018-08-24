use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};
use std::str;
use std::default::Default;
use std::io::{self, Write};
use std::fs::{File};
use std::io::prelude::*;

extern crate bip_bencode;
extern crate hyper;
extern crate sha1;

use bip_bencode::{BencodeRef, BRefAccess, BDecodeOpt};
use hyper::Client;
use hyper::rt::{self, Future, Stream};
use sha1::{Sha1, Digest};


// 
// fn extract_pieces()

fn main() {
    let mut torrent_file: File = File::open("./steven_universe.torrent").unwrap();
    let mut contents = Vec::new();
    let udp = "9.rarbg.me:2710".to_socket_addrs().unwrap();
    println!("{:?}", udp);
    torrent_file.read_to_end(&mut contents);
    // println!("CONTENTS: {:?}", contents);
    let torrent_content = BencodeRef::decode(&contents, BDecodeOpt::default()).unwrap();
    let dict = torrent_content.dict().unwrap();
    let announce_list = dict.lookup(b"announce-list").unwrap();
    let info = dict.lookup(b"info").unwrap();
    let files: Vec<TorrentFile> = info.dict().unwrap().lookup(b"files").unwrap()
        .list().unwrap()
        .clone()
        .into_iter()
        .map(|ref r| extract_file_info(r))
        .collect();

    let piece_length = info.dict().unwrap().lookup(b"piece length").unwrap().int().unwrap();
    let total_size = files
        .iter()
        .fold(0, |acc, ref file| acc + file.length);
    let mut sh = Sha1::new();
    let sha_hash = info.dict().unwrap().lookup(b"pieces").unwrap().bytes().unwrap();
    sh.update(sha_hash);
    let piece_count = (total_size / piece_length) + 1; // this isn't exact. there could always be a payload that is exactly n * piece_length
    let mut torrent: Torrent = Torrent {
        files: files,
        torrent_filename: String::from(info.dict().unwrap().lookup(b"name").unwrap().str().unwrap()),
        piece_length: piece_length,
        pieces: create_empty_pieces((total_size / piece_length) as usize, piece_count as usize),
        sha_hash: sh.digest().to_string()
    };

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
