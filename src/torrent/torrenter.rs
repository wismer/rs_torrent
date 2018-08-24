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
