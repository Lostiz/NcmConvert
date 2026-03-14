use aes::cipher::{BlockDecrypt, KeyInit};
use aes::Aes128Dec;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

const CORE_KEY: [u8; 16] = [
    0x68, 0x7A, 0x48, 0x52, 0x41, 0x6D, 0x73, 0x6F,
    0x35, 0x6B, 0x49, 0x6E, 0x62, 0x61, 0x78, 0x57,
];

const MODIFY_KEY: [u8; 16] = [
    0x23, 0x31, 0x34, 0x6C, 0x6A, 0x6B, 0x5F, 0x21,
    0x5C, 0x5D, 0x26, 0x30, 0x55, 0x3C, 0x27, 0x28,
];

pub struct NeteaseCrypt {
    file_path: PathBuf,
    format: String,
    key_box: [u8; 256],
    audio_data: Vec<u8>,
}

impl NeteaseCrypt {
    pub fn new(file_path: &str) -> Result<Self, String> {
        let path = PathBuf::from(file_path);
        let mut file = File::open(&path).map_err(|e| format!("无法打开文件: {}", e))?;
        
        let mut header = [0u8; 8];
        file.read_exact(&mut header)
            .map_err(|e| format!("读取文件头失败: {}", e))?;
        
        if &header[0..4] != b"CTEN" {
            return Err("不是有效的 NCM 文件".to_string());
        }
        
        file.read_exact(&mut [0u8; 2])
            .map_err(|e| format!("跳过头部失败: {}", e))?;
        
        let mut key_data_len_bytes = [0u8; 4];
        file.read_exact(&mut key_data_len_bytes)
            .map_err(|e| format!("读取密钥长度失败: {}", e))?;
        let key_data_len = u32::from_le_bytes(key_data_len_bytes) as usize;
        
        if key_data_len == 0 {
            return Err("无效的 NCM 文件".to_string());
        }
        
        let mut key_data = vec![0u8; key_data_len];
        file.read_exact(&mut key_data)
            .map_err(|e| format!("读取密钥数据失败: {}", e))?;
        
        for i in 0..key_data.len() {
            key_data[i] ^= 0x64;
        }
        
        let decrypted_key = aes_decrypt_ecb(&CORE_KEY, &key_data)?;
        if decrypted_key.len() < 18 {
            return Err("密钥数据无效".to_string());
        }
        let key = &decrypted_key[17..];
        
        let key_box = build_key_box(key);
        
        let mut meta_len_bytes = [0u8; 4];
        file.read_exact(&mut meta_len_bytes)
            .map_err(|e| format!("读取元数据长度失败: {}", e))?;
        let meta_len = u32::from_le_bytes(meta_len_bytes) as usize;
        
        let format = if meta_len > 0 {
            let mut meta_data = vec![0u8; meta_len];
            file.read_exact(&mut meta_data)
                .map_err(|e| format!("读取元数据失败: {}", e))?;
            
            for i in 0..meta_data.len() {
                meta_data[i] ^= 0x63;
            }
            
            if meta_data.len() > 22 {
                let base64_data = &meta_data[22..];
                let base64_str = String::from_utf8_lossy(base64_data);
                
                if let Ok(decoded) = base64_decode(&base64_str) {
                    if let Ok(decrypted_meta) = aes_decrypt_ecb(&MODIFY_KEY, &decoded) {
                        if decrypted_meta.len() > 6 {
                            let json_str = String::from_utf8_lossy(&decrypted_meta[6..]);
                            if let Some(fmt) = parse_format(&json_str) {
                                fmt
                            } else {
                                "mp3".to_string()
                            }
                        } else {
                            "mp3".to_string()
                        }
                    } else {
                        "mp3".to_string()
                    }
                } else {
                    "mp3".to_string()
                }
            } else {
                "mp3".to_string()
            }
        } else {
            "mp3".to_string()
        };
        
        file.read_exact(&mut [0u8; 5])
            .map_err(|e| format!("跳过CRC失败: {}", e))?;
        
        let mut cover_frame_len_bytes = [0u8; 4];
        file.read_exact(&mut cover_frame_len_bytes)
            .map_err(|e| format!("读取封面帧长度失败: {}", e))?;
        let cover_frame_len = u32::from_le_bytes(cover_frame_len_bytes) as usize;
        
        let mut image_len_bytes = [0u8; 4];
        file.read_exact(&mut image_len_bytes)
            .map_err(|e| format!("读取图片长度失败: {}", e))?;
        let image_len = u32::from_le_bytes(image_len_bytes) as usize;
        
        if image_len > 0 {
            let mut skip_img = vec![0u8; image_len];
            file.read_exact(&mut skip_img)
                .map_err(|e| format!("跳过图片失败: {}", e))?;
        }
        
        if cover_frame_len > image_len && cover_frame_len > 0 {
            let extra_len = cover_frame_len - image_len;
            if extra_len > 0 {
                let mut skip_extra = vec![0u8; extra_len];
                file.read_exact(&mut skip_extra)
                    .map_err(|e| format!("跳过额外数据失败: {}", e))?;
            }
        }
        
        let mut audio_data = Vec::new();
        file.read_to_end(&mut audio_data)
            .map_err(|e| format!("读取音频数据失败: {}", e))?;
        
        for i in 0..audio_data.len() {
            let j = (i + 1) & 0xff;
            let idx = (key_box[j] as usize + key_box[(key_box[j] as usize + j) & 0xff] as usize) & 0xff;
            audio_data[i] ^= key_box[idx];
        }
        
        let detected_format = if audio_data.len() >= 3 {
            if audio_data[0] == 0x49 && audio_data[1] == 0x44 && audio_data[2] == 0x33 {
                "mp3"
            } else {
                "flac"
            }
        } else {
            &format
        };
        
        Ok(NeteaseCrypt {
            file_path: path,
            format: detected_format.to_string(),
            key_box,
            audio_data,
        })
    }
    
    pub fn dump(&self, output_dir: &PathBuf) -> Result<(String, PathBuf), String> {
        let file_stem = self.file_path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        
        let output_filename = format!("{}.{}", file_stem, self.format);
        let output_path = output_dir.join(&output_filename);
        
        let mut output_file = File::create(&output_path)
            .map_err(|e| format!("创建输出文件失败: {}", e))?;
        output_file.write_all(&self.audio_data)
            .map_err(|e| format!("写入音频数据失败: {}", e))?;
        
        Ok((self.format.clone(), output_path))
    }
}

fn aes_decrypt_ecb(key: &[u8; 16], data: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes128Dec::new_from_slice(key)
        .map_err(|e| format!("初始化AES失败: {}", e))?;
    
    let mut result = data.to_vec();
    let block_size = 16;
    
    let n = result.len() >> 4;
    
    for i in 0..n {
        let offset = i << 4;
        if offset + block_size <= result.len() {
            let mut block_arr = [0u8; 16];
            block_arr.copy_from_slice(&result[offset..offset + block_size]);
            cipher.decrypt_block((&mut block_arr).into());
            result[offset..offset + block_size].copy_from_slice(&block_arr);
        }
    }
    
    if !result.is_empty() {
        let pad = result[result.len() - 1];
        if pad > 0 && pad <= 16 {
            let truncate_len = result.len() - pad as usize;
            result.truncate(truncate_len);
        }
    }
    
    Ok(result)
}

fn build_key_box(key: &[u8]) -> [u8; 256] {
    let mut key_box = [0u8; 256];
    for i in 0..256 {
        key_box[i] = i as u8;
    }
    
    let mut last_byte: u8 = 0;
    let mut key_offset: usize = 0;
    
    for i in 0..256 {
        let swap = key_box[i];
        let c = (swap.wrapping_add(last_byte).wrapping_add(key[key_offset])) & 0xff;
        key_offset = (key_offset + 1) % key.len();
        key_box[i] = key_box[c as usize];
        key_box[c as usize] = swap;
        last_byte = c;
    }
    
    key_box
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let input = input.trim();
    let input: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    
    const DECODE_TABLE: [i8; 128] = [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 62, -1, -1, -1, 63,
        52, 53, 54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, -1, -1, -1,
        -1,  0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14,
        15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1, -1, -1,
        -1, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, -1, -1, -1, -1, -1,
    ];
    
    let mut result = Vec::with_capacity(input.len() * 3 / 4);
    
    let chars: Vec<u8> = input.bytes().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let mut accum: u32 = 0;
        let mut count = 0;
        
        for j in 0..4 {
            if i + j < chars.len() {
                let c = chars[i + j];
                if c == b'=' {
                    break;
                }
                if c < 128 {
                    let val = DECODE_TABLE[c as usize];
                    if val >= 0 {
                        accum = (accum << 6) | (val as u32);
                        count += 1;
                    }
                }
            }
        }
        
        if count >= 2 {
            result.push(((accum >> 10) & 0xFF) as u8);
        }
        if count >= 3 {
            result.push(((accum >> 4) & 0xFF) as u8);
        }
        if count >= 4 {
            result.push(((accum << 2) & 0xFF) as u8);
        }
        
        i += 4;
    }
    
    Ok(result)
}

fn parse_format(json_str: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(json_str).ok()?;
    let music = json.get("music")?;
    music.get("format")?.as_str().map(|s| s.to_string())
}
