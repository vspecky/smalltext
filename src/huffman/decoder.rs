use std::{
    collections::HashMap,
    path::Path,
    io::{
        prelude::*,
        BufReader,
        BufWriter,
        Error,
    },
    fs::{
        File,
        OpenOptions,
    },
};

pub struct Decoder {
    file: String,
    table: HashMap<String, u8>,
}

impl Decoder {
    pub fn new(f_name: String) -> Result<Self, &'static str> {
        let path = Path::new(&f_name);

        if !path.exists() || !path.is_file() {
            Err("File not found")
        } else {
            Ok(Self {
                file: f_name,
                table: HashMap::new(),
            })
        }
    }

    fn parse_table(&mut self) -> Result<u32, ()> {
        let f = File::open(&self.file).unwrap();
        let reader = BufReader::new(f);

        let mut read_iter = reader.bytes();

        let mut total_elems = 0_u32;

        for i in 0..3 {
            let byte = unwrap_iter(read_iter.next())?;

            if byte != 0xF0 + i {
                return Err(());
            }
        }

        for i in 0..4 {
            let byte = unwrap_iter(read_iter.next())?;

            total_elems |= (byte as u32) << (i * 8);
        }

        let mut bytes_read = 7_u32;

        let mut elems_parsed = 0_u32;

        while elems_parsed < total_elems {
            let mut code = String::new();
            let byte_val = unwrap_iter(read_iter.next())?;

            let offset = unwrap_iter(read_iter.next())?;
            let amt = unwrap_iter(read_iter.next())?;

            let first = unwrap_iter(read_iter.next())?;
            bytes_read += 4;

            for i in (0..offset).rev() {
                let bit = if (1 << i) & first > 0 {
                    1
                } else {
                    0
                };

                code.push_str(&format!("{}", bit));
            }

            if amt > 1 {
                for _ in 0..(amt - 1) {
                    let byte = unwrap_iter(read_iter.next())?;
                    bytes_read += 1;
                    for j in (0..8).rev() {
                        let bit = if (1 << j) & byte > 0 {
                            1
                        } else {
                            0
                        };
                        code.push_str(&format!("{}", bit));
                    }
                }
            }

            self.table.insert(code, byte_val);
            elems_parsed += 1;
        }

        Ok(bytes_read)
    }

    pub fn decompress(&mut self) -> Result<(), &'static str> {
        if !self.file.ends_with(".cmpr") {
            return Err("File should have the '.cmpr' extension");
        }

        let table_size = match self.parse_table() {
            Ok(size) => size,
            Err(_) => return Err("Error parsing the code table"),
        };

        let in_file = File::open(&self.file).unwrap();
        let reader = BufReader::new(in_file);

        let out_file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.file[..(self.file.len() - 5)])
                    .unwrap();

        let mut writer = BufWriter::new(out_file);

        let mut read_iter = reader.bytes();

        for _ in 0..table_size {
            read_iter.next();
        }

        let mut bit_str = String::new();
        let mut buff = [0_u8];
        for byte_result in read_iter {
            let byte = match byte_result {
                Ok(res) => res,
                Err(_) => return Err("Problem decompressing file")
            };

            for i in (0..8).rev() {
                bit_str.push_str(&format!("{}", (byte >> i) & 0x01));

                if let Some(b) = self.table.get(&bit_str) {
                    buff[0] = *b;
                    if let Err(_) = writer.write(&buff) {
                        return Err("Error decompressing file");
                    };
                    bit_str = String::new();
                }
            }
        }

        if let Err(_) = writer.flush() {
            return Err("Error flushing the buffer");
        }
        
        Ok(())
    }
}

fn unwrap_iter<T>(elem: Option<Result<T, Error>>) -> Result<T, ()> {
    if let Some(res) = elem {
        if let Ok(val) = res {
            Ok(val)
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}