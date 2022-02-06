use crate::emu;
use crate::emu::winapi32::helper;

/*
use crate::emu::context32;
use crate::emu::constants;
use crate::emu::console;

use lazy_static::lazy_static; 
use std::sync::Mutex;*/

// a in RCX, b in RDX, c in R8, d in R9, f then e pushed on stack

pub fn gateway(addr:u64, emu:&mut emu::Emu) {
    match addr {
        0x76dc7070 => LoadLibraryA(emu),
        0x76dd3690 => GetProcAddress(emu),
        0x76db21e0 => CreateToolhelp32Snapshot(emu),
        0x76e0fdb0 => Process32First(emu),
        0x76e0fcc0 => Process32Next(emu),
        0x76db40a0 => LStrCmpI(emu),
        0x76dfc5d0 => AreFileApiIsAnsi(emu),
        0x76e3e420 => BeginUpdateResourceA(emu),
        _ => panic!("calling unimplemented kernel32 API 0x{:x}", addr),
    }
}

fn LoadLibraryA(emu:&mut emu::Emu) {
    let dllptr = emu.regs.rcx;
    let dll = emu.maps.read_string(dllptr);

    match dll.to_lowercase().as_str() {
        "ntdll"|"ntdll.dll" => emu.regs.rax = emu.maps.get_mem("ntdll_pe").get_base(),
        "ws2_32"|"ws2_32.dll" => emu.regs.rax = emu.maps.get_mem("ws2_32_pe").get_base(),
        "wininet"|"wininet.dll" => emu.regs.rax = emu.maps.get_mem("wininet_pe").get_base(),
        "advapi32"|"advapi32.dll" => emu.regs.rax = emu.maps.get_mem("advapi32_pe").get_base(),
        "kernel32"|"kernel32.dll" => emu.regs.rax = emu.maps.get_mem("kernel32_pe").get_base(),
        _ => unimplemented!("/!\\ kernel32!LoadLibraryA: lib not found {}", dll),
    }

    println!("{}** {} kernel32!LoadLibraryA  '{}' =0x{:x} {}", emu.colors.light_red, emu.pos, dll, emu.regs.rax, emu.colors.nc);
}

fn GetProcAddress(emu:&mut emu::Emu) {
    let hndl = emu.regs.rcx;
    let func_ptr = emu.regs.rdx;

    let func = emu.maps.read_string(func_ptr).to_lowercase();

    println!("looking for '{}'", func);

    let peb = emu.maps.get_mem("peb");
    let peb_base = peb.get_base();
    let ldr = peb.read_qword(peb_base + 0x18);
    //println!("ldr: 0x{:x}", ldr);
    let mut flink = emu.maps.read_qword(ldr + 0x10).expect("kernel32!GetProcAddress error reading flink");
    //println!("flink: 0x{:x}", flink);

    loop { // walk modules

        let mod_name_ptr = emu.maps.read_qword(flink + 0x60).expect("kernel32!GetProcAddress error reading mod_name_ptr");
        let mod_path_ptr = emu.maps.read_qword(flink + 0x50).expect("kernel32!GetProcAddress error reading mod_name_ptr");
        //println!("mod_name_ptr: 0x{:x}", mod_name_ptr);

        let mod_base = emu.maps.read_qword(flink + 0x30).expect("kernel32!GetProcAddress error reading mod_addr");
        //println!("mod_base: 0x{:x}", mod_base);

        let mod_name = emu.maps.read_wide_string(mod_name_ptr);
        //println!("mod_name: {}", mod_name);
    

        let pe_hdr_off = match emu.maps.read_dword(mod_base + 0x3c) { 
            Some(hdr) => hdr as u64,
            None => { emu.regs.rax = 0; return; }
        };

        // pe_hdr correct

        
        let export_table_rva = emu.maps.read_dword(mod_base + pe_hdr_off + 0x88).expect("kernel32!GetProcAddress error reading export_table_rva") as u64;
        //println!("({:x}) {:x} =  {:x} + pehdr:{:x} + {:x}", export_table_rva, mod_base + pe_hdr_off + 0x78, mod_base, pe_hdr_off, 0x78);

        if export_table_rva == 0 {
            flink = emu.maps.read_qword(flink).expect("kernel32!GetProcAddress error reading next flink") as u64;
            //println!("getting new flink: 0x{:x}", flink);
            continue;
        }

        let export_table = export_table_rva + mod_base;
        //println!("export_table: 0x{:x}", export_table);

       

        if !emu.maps.is_mapped(export_table) {
            flink = emu.maps.read_qword(flink).expect("kernel32!GetProcAddress error reading next flink") as u64;
            println!("getting new flink: 0x{:x}", flink);
            continue;
        }


        let mut num_of_funcs = emu.maps.read_dword(export_table + 0x18).expect("kernel32!GetProcAddress error reading the num_of_funcs") as u64;
 
        //println!("num_of_funcs:  0x{:x} -> 0x{:x}", export_table + 0x18, num_of_funcs);


        if num_of_funcs == 0 {
            flink = emu.maps.read_qword(flink).expect("kernel32!GetProcAddress error reading next flink") as u64;
            println!("getting new flink: 0x{:x}", flink);
            continue;
        }
        

        let func_name_tbl_rva = emu.maps.read_dword(export_table + 0x20).expect("kernel32!GetProcAddress  error reading func_name_tbl_rva") as u64;
        let func_name_tbl = func_name_tbl_rva + mod_base;

        if num_of_funcs == 0 {
            flink = emu.maps.read_dword(flink).expect("kernel32!GetProcAddress error reading next flink") as u64;
            continue;
        }

        loop { // walk functions
                
            num_of_funcs -= 1;
            let func_name_rva = emu.maps.read_dword(func_name_tbl + num_of_funcs * 4).expect("kernel32!GetProcAddress error reading func_rva") as u64;
            let func_name_va = func_name_rva + mod_base;
            let func_name = emu.maps.read_string(func_name_va).to_lowercase();

            //println!("func_name: {}", func_name);
            
            if func_name == func { 
                let ordinal_tbl_rva = emu.maps.read_dword(export_table + 0x24).expect("kernel32!GetProcAddress error reading ordinal_tbl_rva") as u64;
                let ordinal_tbl = ordinal_tbl_rva + mod_base;
                let ordinal = emu.maps.read_word(ordinal_tbl + 2 * num_of_funcs).expect("kernel32!GetProcAddress error reading ordinal") as u64;
                let func_addr_tbl_rva = emu.maps.read_dword(export_table + 0x1c).expect("kernel32!GetProcAddress  error reading func_addr_tbl_rva") as u64;
                let func_addr_tbl = func_addr_tbl_rva + mod_base;
                
                let func_rva = emu.maps.read_dword(func_addr_tbl + 4 * ordinal).expect("kernel32!GetProcAddress error reading func_rva") as u64;
                let func_va = func_rva + mod_base;

                emu.regs.rax = func_va;

                println!("{}** {} kernel32!GetProcAddress  `{}!{}` =0x{:x} {}", emu.colors.light_red, emu.pos, mod_name, func_name, emu.regs.get_eax() as u32, emu.colors.nc);
                return;
            }

            if num_of_funcs == 0 {
                break;
            }
        }

        flink = emu.maps.read_dword(flink).expect("kernel32!GetProcAddress error reading next flink") as u64;
    } 
}

fn CreateToolhelp32Snapshot(emu:&mut emu::Emu) {
    let flags = emu.regs.rcx;
    let pid = emu.regs.rdx;

    println!("{}** {} kernel32!CreateToolhelp32Snapshot flags: {:x} pid: {} {}", emu.colors.light_red, emu.pos, flags, pid, emu.colors.nc);
    emu.regs.rax = helper::handler_create();
}

fn Process32First(emu:&mut emu::Emu) {
    let handle = emu.regs.rcx;
    let lppe = emu.regs.rdx;

    println!("{}** {} kernel32!Process32First hndl: {:x} lppe: 0x{:x} {}", emu.colors.light_red, emu.pos, handle, lppe, emu.colors.nc);

    if !helper::handler_exist(handle) {
        emu.regs.rax = 0;
        return;
    }

    emu.maps.write_string(lppe +  36, "smss.exe\x00");

/*

            typedef struct tagPROCESSENTRY32 {
            DWORD     dwSize;                +0
            DWORD     cntUsage;              +4
            DWORD     th32ProcessID;         +8
            ULONG_PTR th32DefaultHeapID;    +12
            DWORD     th32ModuleID;         +16
            DWORD     cntThreads;           +20
            DWORD     th32ParentProcessID;  +24
            LONG      pcPriClassBase;       +28
            DWORD     dwFlags;              +32
            CHAR      szExeFile[MAX_PATH];  +36
            } PROCESSENTRY32;
*/

    emu.regs.rax = 1;
}

fn Process32Next(emu:&mut emu::Emu) {
    let handle = emu.regs.rcx;
    let lppe = emu.regs.rdx;

    println!("{}** {} kernel32!Process32Next hndl: {:x} lppe: 0x{:x} {}", emu.colors.light_red, emu.pos, handle, lppe, emu.colors.nc);

    if !helper::handler_exist(handle) {
        emu.regs.rax = 0;
        return;
    }


    emu.regs.rax = 0; // trigger exit loop
}

fn LStrCmpI(emu:&mut emu::Emu) {
    let sptr1 = emu.regs.rcx;
    let sptr2 = emu.regs.rdx;

    let s1 = emu.maps.read_string(sptr1);
    let s2 = emu.maps.read_string(sptr2);

    if s1 == s2 {
        println!("{}** {} kernel32!lstrcmpi `{}` == `{}` {}", emu.colors.light_red, emu.pos, s1, s2, emu.colors.nc);
        emu.regs.rax = 0;

    } else {
        println!("{}** {} kernel32!lstrcmpi `{}` != `{}` {}", emu.colors.light_red, emu.pos, s1, s2, emu.colors.nc);
        emu.regs.rax = 1;
    }
}

fn AreFileApiIsAnsi(emu:&mut emu::Emu) {
    println!("{}** {} kernel32!AreFileApiIsAnsi {}", emu.colors.light_red, emu.pos, emu.colors.nc);
    emu.regs.rax = 1;
}

fn BeginUpdateResourceA(emu:&mut emu::Emu) {
    let pFileName = emu.regs.rcx;
    let bDeleteExistingResources = emu.regs.rdx;
 
    let filename = emu.maps.read_string(pFileName);

    println!("{}** {} kernel32!BeginUpdateResourceA `{}` {} {}", emu.colors.light_red, emu.pos, filename, bDeleteExistingResources, emu.colors.nc);

    emu.regs.rax = helper::handler_create();
}
