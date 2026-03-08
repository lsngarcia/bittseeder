#!/usr/bin/env python3
"""Verify piece hashes from torrent against the actual file."""
import sys, hashlib

def bencode_end(data, pos):
    c = data[pos]
    if c in (ord('d'), ord('l')):
        pos += 1
        while pos < len(data) and data[pos] != ord('e'):
            pos = bencode_end(data, pos)
        return pos + 1
    elif chr(c).isdigit():
        colon = data.index(ord(':'), pos)
        n = int(data[pos:colon])
        return colon + 1 + n
    elif c == ord('i'):
        return data.index(ord('e'), pos + 1) + 1
    return pos + 1

def read_bstring(data, pos):
    colon = data.index(ord(':'), pos)
    n = int(data[pos:colon])
    start = colon + 1
    return data[start:start+n], start + n

def read_bint(data, pos):
    end = data.index(ord('e'), pos + 1)
    return int(data[pos+1:end]), end + 1

def parse_torrent(path):
    with open(path, 'rb') as f:
        data = f.read()
    pos = 1
    piece_length = None
    pieces = None
    single_length = None
    while pos < len(data) and data[pos] != ord('e'):
        key, pos = read_bstring(data, pos)
        if key == b'info':
            info_start = pos
            info_end = bencode_end(data, pos)
            info = data[info_start:info_end]
            pos = info_end
            ipos = 1
            while ipos < len(info) and info[ipos] != ord('e'):
                ikey, ipos = read_bstring(info, ipos)
                if ikey == b'piece length':
                    piece_length, ipos = read_bint(info, ipos)
                elif ikey == b'pieces':
                    pieces, ipos = read_bstring(info, ipos)
                elif ikey == b'length':
                    single_length, ipos = read_bint(info, ipos)
                else:
                    ipos = bencode_end(info, ipos)
        else:
            pos = bencode_end(data, pos)
    return piece_length, pieces, single_length

def verify(torrent_path, file_path, check_pieces=None):
    print(f"Torrent: {torrent_path}")
    print(f"File:    {file_path}")
    piece_length, pieces_raw, file_size = parse_torrent(torrent_path)
    if piece_length is None or pieces_raw is None:
        print("ERROR: Could not parse torrent piece hashes")
        return
    num_pieces = len(pieces_raw) // 20
    print(f"Piece length: {piece_length} bytes ({piece_length//1024} KB)")
    print(f"Pieces: {num_pieces}")
    print(f"File size in torrent: {file_size} bytes")

    with open(file_path, 'rb') as f:
        actual_size = f.seek(0, 2)
    print(f"File size on disk:    {actual_size} bytes")
    if actual_size != file_size:
        print(f"SIZE MISMATCH! Torrent expects {file_size}, disk has {actual_size}")
        return

    if check_pieces is None:
        check_pieces = list(range(num_pieces))

    check_pieces = sorted(set(check_pieces))
    print(f"\nVerifying {len(check_pieces)} pieces...")
    bad = 0
    with open(file_path, 'rb') as f:
        for pi in check_pieces:
            offset = pi * piece_length
            length = min(piece_length, file_size - offset)
            f.seek(offset)
            data = f.read(length)
            computed = hashlib.sha1(data).digest()
            expected = pieces_raw[pi*20:(pi+1)*20]
            if computed != expected:
                bad += 1
                print(f"  piece={pi:5d} offset={offset:12d} len={length:8d}  FAIL  expected={expected.hex()} got={computed.hex()}")
            elif len(check_pieces) <= 20:
                print(f"  piece={pi:5d} offset={offset:12d} len={length:8d}  OK")

    print(f"\nResult: {len(check_pieces)-bad}/{len(check_pieces)} pieces OK")
    if bad:
        print("HASH MISMATCH - file data does not match the torrent piece hashes!")
    else:
        print("All checked pieces match - file is correct.")

if __name__ == '__main__':
    if len(sys.argv) < 3:
        print("Usage: python verify_pieces.py <torrent_file> <data_file>")
        sys.exit(1)
    verify(sys.argv[1], sys.argv[2])
