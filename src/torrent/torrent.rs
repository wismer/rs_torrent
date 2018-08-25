extern crate sha1;
extern crate bip_bencode;

use self::sha1::{Sha1};
use bip_bencode::{BencodeRef, BRefAccess};

const BLOCK_LENGTH: i64 = 16384;

pub struct TorrentFile {
    pub length: i64,
    pub path: Vec<String>
}

pub struct Torrent {
    pub files: Vec<TorrentFile>,
    pub torrent_filename: String,
    pub piece_length: i64,
    pub pieces: Vec<Piece>,
    pub sha_hash: String,
    // tracker_urls: Vec<String>
}

type Block = Option<Vec<u8>>;

pub struct Piece {
    pub blocks: Vec<Block>,
    pub index_range: (usize, usize),
    pub completed: bool
}

impl Torrent {
    pub fn new(bencode_file: &BencodeRef) -> Torrent {
        let dict = bencode_file.dict().unwrap();
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

        Torrent {
            files: files,
            torrent_filename: String::from(info.dict().unwrap().lookup(b"name").unwrap().str().unwrap()),
            piece_length: piece_length,
            pieces: create_empty_pieces((total_size / piece_length) as usize, piece_count as usize),
            sha_hash: sh.digest().to_string()
        }
    }

    fn tracker_query_params(&self) -> Option<String> {
        

        None
    }
}


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
