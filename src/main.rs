use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};
extern crate bip_bencode;
extern crate hyper;


use std::default::Default;
use bip_bencode::{BencodeRef, BRefAccess, BDecodeOpt};
use std::io::{self, Write};
use hyper::Client;
use hyper::rt::{self, Future, Stream};
use std::fs::{File};
use std::io::prelude::*;

const BLOCK_LENGTH: i64 = 16384;

struct TorrentFile {
    pub length: i64,
    pub path: Vec<String>
}

struct Torrent {
    pub files: Vec<TorrentFile>,
    pub torrent_filename: String,
    pub piece_length: i64,
    pub pieces: Vec<Piece>,
    // tracker_urls: Vec<String>
}

type Block = Option<Vec<u8>>;

struct Piece {
    pub blocks: Vec<Block>,
    pub index_range: (usize, usize),
    pub completed: bool
}
// 
// fn extract_pieces()

fn extract_file_info(bencode_ref: &BencodeRef) -> TorrentFile {
    let file_length = bencode_ref.dict().unwrap().lookup(b"length").unwrap().int().unwrap();
    let file_path = bencode_ref.dict().unwrap().lookup(b"path").unwrap().list().unwrap()
        .clone()
        .into_iter()
        .map(|r| String::from(r.str().unwrap()))
        .collect();

    TorrentFile {
        length: file_length,
        path: file_path
    }
}


fn create_empty_pieces(count: usize, piece_length: usize) -> Vec<Piece> {
    let mut pieces: Vec<Piece> = Vec::with_capacity(count);
    
    for i in 0..count {
        let piece: Piece = Piece {
            blocks: vec![None; piece_length / BLOCK_LENGTH as usize],
            index_range: (i * piece_length, (i * piece_length) + piece_length),
            completed: false
        };
        pieces.push(piece);
    }

    pieces
}

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

    let sha_hash = info.dict().unwrap().lookup(b"pieces").unwrap().bytes().unwrap();
    println!("SHA HASH {:?}", sha_hash.len());

    let piece_count = (total_size / piece_length) + 1; // this isn't exact. there could always be a payload that is exactly n * piece_length
    let mut torrent: Torrent = Torrent {
        files: files,
        torrent_filename: String::from(info.dict().unwrap().lookup(b"name").unwrap().str().unwrap()),
        piece_length: piece_length,
        pieces: create_empty_pieces((total_size / piece_length) as usize, piece_count as usize)
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
