use macroquad::color::Color;
use image::GenericImageView;
use colored::Colorize;

pub fn format_bytes(bytes: usize) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes_f64 = bytes as f64;

    if bytes_f64 < KB {
        format!("{} B", bytes)
    } else if bytes_f64 < MB {
        format!("{:.2} KB", bytes_f64 / KB)
    } else if bytes_f64 < GB {
        format!("{:.2} MB", bytes_f64 / MB)
    } else {
        format!("{:.2} GB", bytes_f64 / GB)
    }
}

enum Region {
    Run(u8, usize),
    Literal(Vec<u8>),
}

fn build_regions(data: &[u8]) -> Vec<Region> {
    let mut regions = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Check for run
        if i + 2 < data.len() && data[i] == data[i + 1] && data[i] == data[i + 2] {
            let byte = data[i];
            let mut run_len = 3;
            while i + run_len < data.len() && data[i + run_len] == byte && run_len < 128 {
                run_len += 1;
            }
            regions.push(Region::Run(byte, run_len));
            i += run_len;
        } else {
            let mut literal = Vec::new();
            while i < data.len() {
                // Stop before a run of 3
                if i + 2 < data.len()
                    && data[i] == data[i + 1]
                    && data[i] == data[i + 2]
                {
                    break;
                }
                literal.push(data[i]);
                i += 1;
                if literal.len() == 128 {
                    break;
                }
            }
            regions.push(Region::Literal(literal));
        }
    }

    regions
}

fn compress(data: &[u8]) -> Result<Vec<u8>, MEIFParserError> {
    let regions = build_regions(data);
    let mut result = Vec::new();

    for region in regions {
        match region {
            Region::Run(byte, len) => {
                result.push(0b1000_0000 | ((len - 1) as u8));
                result.push(byte);
            }
            Region::Literal(lit) => {
                result.push((lit.len() - 1) as u8);
                result.extend(lit);
            }
        }
    }

    if decompress(&result)? != data {
        return Err(MEIFParserError::new("Compression failed: data mismatch"));
    }

    Ok(result)
}

fn decompress(data: &[u8]) -> Result<Vec<u8>, MEIFParserError> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let header = data[i];
        i += 1;
        let is_run = (header & 0b1000_0000) != 0;
        let length = (header & 0b0111_1111) as usize + 1;

        if is_run {
            if i >= data.len() {
                // panic!("Unexpected end of input while decoding run");
                return Err(MEIFParserError::new("Unexpected end of input while decoding run"));
            }
            let value = data[i];
            i += 1;
            result.extend(std::iter::repeat(value).take(length));
        } else {
            if i + length > data.len() {
                // panic!("Unexpected end of input while decoding literal");
                return Err(MEIFParserError::new("Unexpected end of input while decoding literal"));
            }
            result.extend_from_slice(&data[i..i + length]);
            i += length;
        }
    }

    Ok(result)
}

#[derive(PartialEq)]
pub struct MEIFImage {
    pub width: u32,
    pub height: u32,
    pub indexes: Vec<Color>,
    pub data: Vec<u8>,
}

impl MEIFImage {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // MEIF header
        bytes.extend(b"MEIF");
        bytes.extend(&[0x00; 4]);
        bytes.extend(&[0x69, 0x42]);
        bytes.extend(&[0x00; 2]);

        // DIMN section
        bytes.extend(b"DIMN");
        let mut fb00 = 0;
        let mut fb01 = 0;
        let mut fb10 = 0;
        let mut fb11 = 0;
        for b01 in 0..32 {
            let b00 = (self.width + b01) / 32;
            if 32 * b00 - b01 == self.width {
                fb00 = b00 as u8;
                fb01 = b01 as u8;
                break;
            }
        }
        for b11 in 0..32 {
            let b10 = (self.height + b11) / 32;
            if 32 * b10 - b11 == self.height {
                fb10 = b10 as u8;
                fb11 = b11 as u8;
                break;
            }
        }
        bytes.push(fb00 as u8);
        bytes.push(fb01 as u8);
        bytes.push(fb10 as u8);
        bytes.push(fb11 as u8);

        // INDX section
        bytes.extend(b"INDX");
        for color in &self.indexes {
            bytes.push((color.r * 255.0) as u8);
            bytes.push((color.g * 255.0) as u8);
            bytes.push((color.b * 255.0) as u8);
        }

        // DATA section with proper RLE
        bytes.extend(b"DATA");
        bytes.extend(compress(&self.data).unwrap());

        // DONE! section
        bytes.extend(b"DONE!");

        bytes
    }

    pub fn to_rgb_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity((self.width * self.height * 3) as usize);
        for i in 0..self.data.len() {
            let idx = self.data[i] as usize;
            let color = &self.indexes[idx];
            bytes.push((color.r * 255.0).round() as u8);
            bytes.push((color.g * 255.0).round() as u8);
            bytes.push((color.b * 255.0).round() as u8);
        }
        bytes
    }
}

impl std::fmt::Debug for MEIFImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MEIFImage {{ width: {}, height: {}, indexes: {} }}", self.width, self.height, self.indexes.len())
    }
}

pub struct MEIFParserError {
    pub message: String,
}

impl MEIFParserError {
    pub fn new(message: &str) -> Self {
        MEIFParserError {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Debug for MEIFParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MEIFParserError: {}", self.message)
    }
}

pub struct MEIFParser {
    pub buffer: Vec<u8>,
    pointer: usize,
}

impl MEIFParser {
    pub fn new(buffer: Vec<u8>) -> Self {
        MEIFParser { buffer, pointer: 0 }
    }

    pub fn fetch(&mut self) -> Result<u8, MEIFParserError> {
        if self.pointer >= self.buffer.len() {
            return Err(MEIFParserError::new("Buffer overflow"));
        }
        let data = self.buffer[self.pointer];
        self.pointer += 1;
        Ok(data)
    }

    pub fn next_bytes(&mut self, bytes: &[u8]) -> bool {
        // Do self.fetch() until we have enough bytes
        if self.pointer + bytes.len() > self.buffer.len() {
            return false;
        }
        for i in 0..bytes.len() {
            if self.fetch().unwrap() != bytes[i] {
                return false;
            }
        }
        true
    }

    pub fn non_advancing_next_bytes(&self, bytes: &[u8]) -> bool {
        // Do self.fetch() until we have enough bytes
        if self.pointer + bytes.len() > self.buffer.len() {
            return false;
        }
        for i in 0..bytes.len() {
            if self.buffer[self.pointer + i] != bytes[i] {
                return false;
            }
        }
        true
    }

    pub fn parse(&mut self) -> Result<MEIFImage, MEIFParserError> {
        let mut meif_image = MEIFImage {
            width: 0,
            height: 0,
            indexes: Vec::new(),
            data: Vec::new(),
        };
        let passed = "==>".green();
        let mut start = std::time::Instant::now();
        if !self.next_bytes(b"MEIF") {
            return Err(MEIFParserError::new("Invalid MEIF header"));
        }
        if !self.next_bytes(&[0x00; 4]) {
            return Err(MEIFParserError::new("Expected 4 0x00 bytes after MEIF header"));
        }
        if !self.next_bytes(&[0x69, 0x42]) {
            return Err(MEIFParserError::new("Expected \"\"\"nice + answer to life\"\"\" after 6 0x00 bytes"));
        }
        if !self.next_bytes(&[0x00; 2]) {
            return Err(MEIFParserError::new("Expected 2 0x00 bytes after \"\"\"nice + answer to life\"\"\""));
        }
        println!("{} MEIF header parsed successfully. Took {}ms", passed, start.elapsed().as_millis());
        start = std::time::Instant::now();
        if !self.next_bytes(b"DIMN") {
            return Err(MEIFParserError::new("Expected \"DIMN\" section"));
        }
        let b00 = self.fetch()?;
        let b01 = self.fetch()?;
        let b10 = self.fetch()?;
        let b11 = self.fetch()?;
        meif_image.width = (b00 as u32 * 32) - (b01 as u32);
        meif_image.height = (b10 as u32 * 32) - (b11 as u32);
        println!("{} DIMN section parsed successfully. Took {}ms", passed, start.elapsed().as_millis());
        start = std::time::Instant::now();
        if !self.next_bytes(b"INDX") {
            return Err(MEIFParserError::new("Expected \"INDX\" section"));
        }
        while !self.non_advancing_next_bytes(b"DATA") {
            let r = self.fetch()? as f32 / 255.0;
            let g = self.fetch()? as f32 / 255.0;
            let b = self.fetch()? as f32 / 255.0;
            meif_image.indexes.push(Color::new(r, g, b, 1.0));
        }
        println!("{} INDX section parsed successfully. Took {}ms", passed, start.elapsed().as_millis());
        
        start = std::time::Instant::now();
        if !self.next_bytes(b"DATA") {
            return Err(MEIFParserError::new("Expected \"DATA\" section"));
        }
        let mut compressed_data: Vec<u8> = Vec::new();
        loop {
            if self.pointer >= self.buffer.len() {
                return Err(MEIFParserError::new("File ended abruptly"));
            }
            if self.non_advancing_next_bytes(b"DONE!") {
                break;
            }
            compressed_data.push(self.fetch()?);
        }
        let data = decompress(&compressed_data)?;
        if data.len() != (meif_image.width * meif_image.height) as usize {
            return Err(MEIFParserError::new(format!("Data length mismatch, got {} but expected {}.", data.len(), meif_image.width * meif_image.height).as_str()));
        }
        for i in 0..data.len() {
            if data[i] as usize >= meif_image.indexes.len() {
                return Err(MEIFParserError::new(format!("Invalid index {} at position {}", data[i], i).as_str()));
            }
        }
        meif_image.data = data;
        println!("{} DATA section parsed successfully. Took {}ms", passed, start.elapsed().as_millis());
        Ok(meif_image)
    }
}

pub struct MEIFConverter {
    pub image: image::DynamicImage,
}

impl MEIFConverter {
    pub fn new(image: image::DynamicImage) -> Self {
        MEIFConverter { image }
    }

    pub fn convert(&self) -> Result<MEIFImage, MEIFParserError> {
        let mut meif_image = MEIFImage {
            width: self.image.width(),
            height: self.image.height(),
            indexes: Vec::new(),
            data: Vec::with_capacity((self.image.width() * self.image.height()) as usize),
        };

        let mut index_map: std::collections::HashMap<Vec<u8>, u8> = std::collections::HashMap::new();
        let mut index_count: u8 = 0;

        for y in 0..self.image.height() {
            for x in 0..self.image.width() {
                let pixel = self.image.get_pixel(x, y);
                let color = vec![pixel[0], pixel[1], pixel[2]];
                if let Some(&index) = index_map.get(&color) {
                    meif_image.data.push(index);
                } else {
                    let mut found_similar = false;

                    for (existing_color, existing_index) in &index_map {
                        const THRESHOLD: u8 = 20;
                        if (pixel[0] as f32 - existing_color[0] as f32).abs() < THRESHOLD as f32
                            && (pixel[1] as f32 - existing_color[1] as f32).abs() < THRESHOLD as f32
                            && (pixel[2] as f32 - existing_color[2] as f32).abs() < THRESHOLD as f32
                        {
                            meif_image.data.push(*existing_index);
                            found_similar = true;
                            break;
                        }
                    }

                    if !found_similar {
                        if index_count >= 255 {
                            // Find the closest color in the index map and use that instead
                            let mut closest_index = 0;
                            let mut closest_distance = u32::MAX;
                            for (existing_color, existing_index) in &index_map {
                                let distance = ((pixel[0] as i32 - existing_color[0] as i32).pow(2)
                                    + (pixel[1] as i32 - existing_color[1] as i32).pow(2)
                                    + (pixel[2] as i32 - existing_color[2] as i32).pow(2)) as u32;
                                if distance < closest_distance {
                                    closest_distance = distance;
                                    closest_index = *existing_index;
                                }
                            }
                            meif_image.data.push(closest_index);
                        } else {
                            index_map.insert(color.clone(), index_count);
                            meif_image.indexes.push(Color::new(
                                pixel[0] as f32 / 255.0,
                                pixel[1] as f32 / 255.0,
                                pixel[2] as f32 / 255.0,
                                1.0,
                            ));
                            meif_image.data.push(index_count);
                            index_count += 1;
                        }
                    }
                }
            }
        }

        Ok(meif_image)
    }
}
