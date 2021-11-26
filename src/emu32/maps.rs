mod mem32;

use mem32::Mem32;
use std::collections::HashMap;


pub struct Maps {
    pub maps: HashMap<String,Mem32>,
}

impl Maps {
    pub fn new() -> Maps {
        Maps {
            maps: HashMap::new(),
        }
    }

    pub fn create_map(&mut self, name:&str) -> &mut Mem32 {
        let mem = Mem32::new();
        self.maps.insert(name.to_string(), mem);
        return self.maps.get_mut(name).expect("incorrect memory map name");
    }

    pub fn write_dword(&mut self, addr:u32, value:u32) -> bool {
        for (_,mem) in self.maps.iter_mut() {
            if mem.inside(addr) {
                mem.write_dword(addr, value);
                return true;
            }
        }
        println!("writing on non mapped zone 0x{:x}", addr);
        return false;
    }

    pub fn write_word(&mut self, addr:u32, value:u16) -> bool {
        for (_,mem) in self.maps.iter_mut() {
            if mem.inside(addr) {
                mem.write_word(addr, value);
                return true;
            }
        }
        println!("writing on non mapped zone 0x{:x}", addr);
        return false;
    }

    pub fn write_byte(&mut self, addr:u32, value:u8) -> bool {
        for (_,mem) in self.maps.iter_mut() {
            if mem.inside(addr) {
                mem.write_byte(addr, value);
                return true;
            }
        }
        println!("writing on non mapped zone 0x{:x}", addr);
        return false;
    }

    pub fn read_dword(&self, addr:u32) -> Option<u32> {
        for (_,mem) in self.maps.iter() {
            if mem.inside(addr) {
                return Some(mem.read_dword(addr));
            }
        }
        return None;
    }

    pub fn read_word(&self, addr:u32) -> Option<u16> {
        for (_,mem) in self.maps.iter() {
            if mem.inside(addr) {
                return Some(mem.read_word(addr));
            }
        }
        return None;
    }

    pub fn read_byte(&self, addr:u32) -> Option<u8> {
        for (_,mem) in self.maps.iter() {
            if mem.inside(addr) {
                return Some(mem.read_byte(addr));
            }
        }
        return None;
    }

    pub fn get_mem(&mut self, name:&str) -> &mut Mem32 {
        return self.maps.get_mut(&name.to_string()).expect("incorrect memory map name");
    }

    pub fn get_mem_by_addr(&self, addr:u32) -> Option<&Mem32> {
        for (_,mem) in self.maps.iter() {
            if mem.inside(addr) {
                return Some(&mem);
            }
        }
        return None;
    }

    pub fn memcpy(&mut self, to:u32, from:u32, size:usize) -> bool {
        let mut b:u8;
        for i in 0..size {
            b = match self.read_byte(from+i as u32) {
                Some(v) => v,
                None => return false,
            };
            if !self.write_byte(to+i as u32, b) {
                return false;
            }
        }
        return true;
    }

    /*
    pub fn memcpy_faster(&mut self, to:u32, from:u32, size:u32) -> bool {
        let mem_from = match self.get_mem_by_addr(from) {
            Some(v) => v,
            None => return false,    
        };
        let mut mem_to = match self.get_mem_by_addr(to) {
            Some(v) => v,
            None => return false,    
        };
        let mut b:u8;

        for i in 0..size as usize  {
            if !mem_from.inside(from+i as u32) {
                return false;
            }
            if !mem_to.inside(to+i as u32) {
                return false;
            }
            b = mem_from.read_byte(from+i as u32);
            mem_to.write_byte(to+i as u32, b);
        }

        return true;
    }*/


    pub fn print_maps(&self) {
        println!("--- maps ---");
        for k in self.maps.keys() {
            let map = self.maps.get(k).unwrap();
            println!("{}\t0x{:x} - 0x{:x}", k, map.get_base(), map.get_bottom());
        }
        println!("memory usage: {} bytes", self.size());
        println!("---");
    }



    pub fn is_mapped(&self, addr:u32) -> bool {
        for (_,mem) in self.maps.iter() {
            if mem.inside(addr) {
                return true;
            }
        }
        return false;
    }

    pub fn get_addr_name(&self, addr:u32) -> Option<String> {
        for (name,mem) in self.maps.iter() {
            if mem.inside(addr) {
                return Some(name.to_string());
            }
        }
        return None;
    }

    pub fn dump(&self, addr:u32) {
        let mut count = 0;
        for _ in 0..8 {
            let mut bytes:Vec<char> = Vec::new();
            for _ in 0..4 {
                let dw = match self.read_dword(addr + count*4) {
                    Some(v) => v,
                    None => {
                        println!("bad address");
                        return;
                    }
                };
                count += 1;
                bytes.push(((dw&0xff) as u8) as char);
                bytes.push((((dw&0xff00)>>8) as u8) as char);
                bytes.push((((dw&0xff0000)>>16) as u8) as char);
                bytes.push((((dw&0xff000000)>>24) as u8) as char);
                print!("{:02x} {:02x} {:02x} {:02x}  ", dw&0xff, (dw&0xff00)>>8, (dw&0xff0000)>>16, (dw&0xff000000)>>24);
            }
            //let s:String = String::from_iter(bytes);
            //let s = str::from_utf8(&bytes).unwrap();
            let s: String = bytes.into_iter().collect();
            println!("{}",s);
        }
    }

    pub fn read_bytes(&self, addr:u32, sz:usize) -> &[u8] {
        let mem = match self.get_mem_by_addr(addr) {
            Some(v) => v,
            None => return &[0;0],
        };
        let bytes = mem.read_bytes(addr, sz);
        return bytes;
    }

    pub fn read_string_of_bytes(&self, addr:u32, sz:usize) -> String {
        let mut svec:Vec<String> = Vec::new();
        let bytes = self.read_bytes(addr, sz);
        for i in 0..bytes.len() {   
            svec.push(format!("{:02x} ", bytes[i]));
        }
        let s:String = svec.into_iter().collect();
        return s;
    }



    pub fn read_string(&self, addr:u32) -> String {
        let mut bytes:Vec<char> = Vec::new();
        let mut b:u8;
        let mut i:u32 = 0;

        loop {
            b = match self.read_byte(addr+i) {
                Some(v) => v,
                None => break,
            };
            
            if b == 0x00 {
                break;
            }

            i += 1;
            bytes.push(b as char);
        }

        let s: String = bytes.into_iter().collect();
        return s;
    }

    pub fn read_wide_string(&self, addr:u32) -> String {
        let mut bytes:Vec<char> = Vec::new();
        let mut b:u8;
        let mut i:u32 = 0;

        loop {
            b = match self.read_byte(addr+i) {
                Some(v) => v,
                None => break,
            };
            
            if b == 0x00 {
                break;
            }

            i += 2;
            bytes.push(b as char);
        }

        let s: String = bytes.into_iter().collect();
        return s;
    }

    pub fn search_string(&self, kw:&String, map_name:&String) -> bool {
        let mut found:bool = false;

        for (name,mem) in self.maps.iter() {
            if name == map_name {
                for addr in mem.get_base()..mem.get_bottom() {
                    let bkw = kw.as_bytes();
                    let mut c = 0;
                    
                    for i in 0..bkw.len() {
                        let b = mem.read_byte(addr+(i as u32));
                        if b == bkw[i] {
                            c+=1;
                        } else {
                            break;
                        }
                    }

                    if c == bkw.len() {
                        println!("found at 0x{:x}", addr);
                        found = true;
                    }

                }
                if !found {
                    println!("string not found.");
                }
                return found;
            }
        }
        println!("map not found");
        return false;
    }

    pub fn write_spaced_bytes(&mut self, addr:u32, sbs:String) -> bool {
        let bs:Vec<&str> = sbs.split(" ").collect();
        for i in 0..bs.len() {
            let b = u8::from_str_radix(bs[i],16).expect("bad num conversion");
            if !self.write_byte(addr, b) {
                return false;
            }
        }
        return true;
    }

    pub fn search_spaced_bytes(&self, sbs:&String, map_name:&String) -> bool {
        let bs:Vec<&str> = sbs.split(" ").collect();
        let mut bytes:Vec<u8> = Vec::new();
        for i in 0..bs.len() {
            let b = match u8::from_str_radix(bs[i],16) {
                Ok(b) => b,
                Err(_) => {
                    println!("bad hex bytes");
                    return false;
                }
            };
            bytes.push(b);
        }
        return self.search_bytes(bytes, map_name);
    }

    pub fn search_space_bytes_in_all(&self, sbs:String)  -> bool {
        let mut found:bool = false;
        for (name,_) in self.maps.iter() {
            if self.search_spaced_bytes(&sbs, name) {
                found = true;
            }
        }
        return found;
    }

    pub fn search_string_in_all(&self, kw:String) {
        for (name,_) in self.maps.iter() {
            self.search_string(&kw, name);
        }
    }
   
    pub fn search_bytes(&self, bkw:Vec<u8>, map_name:&String) -> bool {
        let mut found:bool = false;

        for (name,mem) in self.maps.iter() {
            if name == map_name {
                for addr in mem.get_base()..mem.get_bottom() {
                    let mut c = 0;
                    
                    for i in 0..bkw.len() {
                        let b = mem.read_byte(addr+(i as u32));
                        if b == bkw[i] {
                            c+=1;
                        } else {
                            break;
                        }
                    }

                    if c == bkw.len() {
                        println!("found at 0x{:x}", addr);
                        found = true;
                    }

                }

                return found;
            }
        }
        println!("map not found");
        return false;
    }

    pub fn size(&self) -> usize {
        let mut sz:usize = 0;
        for (_,mem) in self.maps.iter() {
            sz += mem.size();
        }
        return sz;
    }

    pub fn alloc(&self, sz:u32) -> Option<u32> {
        // super simple memory allocator

        let mut addr:u32 = 0;

        //println!("ALLOCATOR sz:{}", sz);
        loop {
            addr += sz;
            //println!("trying 0x{:x}", addr);
            
            if addr >= 0x70000000 {
                return None;
            }

            //println!("step1");

            for (_,mem) in self.maps.iter() {
                if addr >= mem.get_base() && addr <= mem.get_bottom() {
                    continue;
                }
            }

            //println!("step2");

            if !self.is_mapped(addr) {
                return Some(addr);
            }

        }
    }

    pub fn save(&self, addr:u32, size:u32, filename:String) {
        match self.get_mem_by_addr(addr) {
            Some(m) => {
                m.save(addr, size as usize, filename);
            },
            None => {
                println!("this address is not mapped.");
            }
        }
    }

}


