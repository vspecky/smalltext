use std::{
    io::{
        BufReader,
        BufWriter,
        prelude::*,
        Error,
    },
    fs::{
        File,
        OpenOptions,
    },
    collections::HashMap,
    path::Path,
    vec::Vec,
};

type Child = Option<Box<Node>>;

struct Node {
    ch: Option<u8>,
    freq: u32,
    left: Child,
    right: Child,
}

pub struct Encoder {
    freq: HashMap<u8, u32>,
    file: String,
    table: HashMap<u8, String>,
}

impl Node {
    fn box_new(ch: Option<u8>, freq: u32) -> Box<Self> {
        Box::new(Self {
            ch, freq,
            left: None,
            right: None
        })
    }

    fn set_left(&mut self, l: Box<Node>) {
        self.left = Some(l);
    }

    fn set_right(&mut self, r: Box<Node>) {
        self.right = Some(r);
    }
}

impl Encoder {
    pub fn new(f_name: String) -> Result<Self, &'static str> {
        let path = Path::new(&f_name);

        if !path.exists() || !path.is_file() {
            return Err("Problem reading file");
        }

        Ok(Self {
            freq: HashMap::new(),
            file: f_name,
            table: HashMap::new(),
        })
    }

    fn get_frequency(&mut self) -> Result<(), Error> {
        let f = File::open(&self.file)?;
        
        let buf = BufReader::new(f);

        for byte in buf.bytes() {
            let cntr = self.freq.entry(byte.unwrap()).or_insert(0);

            *cntr += 1;
        }

        Ok(())
    }

    fn get_tree(&mut self) -> Child {
        let mut nodes: Vec<Box<Node>> = self.freq
            .iter()
            .map(|n| Node::box_new(Some(*n.0), *n.1))
            .collect();

        while nodes.len() > 1 {
            nodes.sort_by(|a, b| b.freq.cmp(&a.freq));
            let left = nodes.pop().unwrap();
            let right = nodes.pop().unwrap();

            let mut parent = Node::box_new(None, left.freq + right.freq);
            parent.set_left(left);
            parent.set_right(right);
            nodes.push(parent);
        }

        Some(nodes.pop().unwrap())
    }

    fn code_table(&mut self, node: Box<Node>, code: String) {
        match node.ch {
            Some(b) => { self.table.insert(b, code); },
            None => {
                if let Some(l) = node.left {
                    self.code_table(l, code.clone() + "0");
                }

                if let Some(r) = node.right {
                    self.code_table(r, code.clone() + "1");
                }
            }
        };
    }

    fn serialize_table(&mut self) -> Vec<u8> {
        let mut table: Vec<u8> = vec![];
        let mut total_elems = 0_u32;

        table.push(0xF0);
        table.push(0xF1);
        table.push(0xF2);

        for (key, value) in self.table.iter() {
            table.push(*key);
            let rem = value.len() % 8;
            let code_len = value.len() / 8 + if rem > 0 { 1 } else { 0 };
            total_elems += 1;

            table.push(rem as u8);
            table.push(code_len as u8);
            if rem != 0 {
                let first_byte = u8::from_str_radix(&value[..rem], 2).unwrap();
                table.push(first_byte);

                if code_len > 1 {
                    let bytes = to_byte_array(&value[rem..]);
                    for b in bytes.into_iter() {
                        table.push(b);
                    }
                }

            } else {
                let bytes = to_byte_array(&value);
                for b in bytes.into_iter() {
                    table.push(b);
                }
            }
        }

        for i in 0..4 {
            let val = ((total_elems >> ((3 - i) * 8)) & 0xFF) as u8;
            table.insert(3, val);
        }

        table
    }

    pub fn compress(&mut self) -> Result<(), Error> {
        self.get_frequency()?;
        let tree = self.get_tree();
        self.code_table(tree.unwrap(), String::from(""));

        let orig_file = File::open(&self.file)?;

        let reader = BufReader::new(orig_file);

        let new_file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&(self.file.clone() + ".cmpr"))?;

        let mut writer = BufWriter::new(new_file);

        writer.write(&self.serialize_table())?;

        let mut byte_str = String::new();

        for byte in reader.bytes() {
            if let Some(code) = self.table.get(&byte.unwrap()) {
                byte_str.push_str(&code);
            }

            if byte_str.len() < 8 { continue }

            let rem = byte_str.len() % 8;
            let bytes = to_byte_array(&byte_str[0..(byte_str.len() - rem)]);
            writer.write(&bytes)?;
            byte_str = String::from(&byte_str[(byte_str.len() - rem)..byte_str.len()]);
        }

        for _ in 0..(8 - byte_str.len()) {
            byte_str.push_str("0");
        }

        writer.write(&to_byte_array(&byte_str))?;

        writer.flush()?;

        Ok(())
    }
}

fn to_byte_array(s: &str) -> Vec<u8> {
    let mut res = Vec::new();

    for i in 0..(s.len() / 8) {
        res.push(u8::from_str_radix(&s[(i * 8)..((i + 1) * 8)], 2).unwrap());
    }

    res
}