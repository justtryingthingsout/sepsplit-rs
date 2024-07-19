/*
    sepsplit-rs - A tool to split SEPOS firmware into its individual modules
    Copyright (C) 2024 plzdonthaxme

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

/*
    This file is built on some assumptions that may be wrong.
    Namely:
        - That the SEP HDR struct will not have 3 
          u64 fields before the rootserver if the 
          Shared Memory Entry size is 0 (N142b SEP)
        - That the SEP HDR and SEP App 64-bit structs 
          will not have memory sizes if the stack size is 0
          (iOS 13 SEP)
    If these assumptions are wrong, the code may panic due to the struct fields being off.
*/
#![allow(dead_code)] // fields kept for documentation

use binrw::{BinRead, binrw};

//utility macros/functions to help make my life easier

//generate a struct from a slice of bytes, using binread
#[macro_export]
macro_rules! cast_struct {
    ($t: ty, $arr: expr) => {
        Cursor::new(&$arr).read_le::<$t>().unwrap_or_else(|e| panic!("Unable to deserialize to {}, err: {e}", stringify!($t)))
    }
}

#[macro_export]
macro_rules! write_struct {
    ($str: expr, $arr: expr) => {
        Cursor::new(&mut $arr).write_le(&$str).unwrap_or_else(|e| panic!("Unable to serialize {}, err: {e}", stringify!($str)))
    }
}

//generate a struct from a slice of bytes with imported arguments, using binrw
#[macro_export]
macro_rules! cast_struct_args {
    ($t: ty, $arr: expr, $args: expr) => {
        <$t>::read_le_args(&mut Cursor::new($arr), $args)
        .unwrap_or_else(|e|
            panic!(
                "Unable to deserialize to {}, err: {e}, first 4 bytes: {bytes:x?}", 
                stringify!($t), 
                bytes=&$arr[0..4]
            )
        )
    }
}

//create a range from the start and size
pub const fn range_size(start: usize, size: usize) -> std::ops::Range<usize> {
    start..start+size
}

//make a str from a slice
pub fn strslice(slice: &[u8]) -> &str {
    std::str::from_utf8(slice).unwrap_or_else(|e| 
        panic!("Could not convert slice to utf-8, err: {e}")
    ).split_whitespace().next().unwrap()
}

//write to file with a buffer
pub fn filewrite(path: &std::path::Path, data: &[u8]) {
    use std::io::Write;
    std::io::BufWriter::new(std::fs::File::create(path).unwrap_or_else(|e| 
        panic!("Unable to create \"{path}\" with err: {e}", path=path.display())
    )).write_all(data).unwrap_or_else(|e| 
        panic!("Unable to write to buffered file \"{path}\" with err: {e}", path=path.display())
    );
}

//structs

#[derive(BinRead)]
 pub struct Legion64 {
    _unk1: u64,
    _uuidtext: [u8; 4],
    _unk2: u64,
    _unk3: u32,
    pub uuid: [u8; 16],
    _unk4: u64,
    _unk5: u64,
    pub subversion: u32, //0x4
    pub legionstr: [u8; 16],
    pub structoff: u16,
    _reserved: [u8; 2]
} 

#[derive(BinRead)]
 pub struct Legion64Old {
    pub subversion: u32, //0x3
    pub legionstr: [u8; 16],
    pub structoff: u16,
    _reserved: [u8; 2]
} 

#[derive(BinRead)]
pub struct Legion32 {
    pub subversion: u32, //0x1
    pub off: u32, //0x800
    pub legionstr: [u8; 16]
}

//all of the below allows are due to macro generated code
#[allow(
    dead_code, 
    clippy::map_unwrap_or, 
    clippy::no_effect_underscore_binding, 
    clippy::cast_lossless
)]
mod srcver { //in a module to be able to apply allow attribute
    use modular_bitfield::prelude::*;
    use ::binrw::BinRead;
    use std::fmt;
    
    #[bitfield(bits = 64)]
    #[derive(BinRead, Debug, Clone, Copy)]
    #[br(map = Self::from_bytes)]
    pub struct SrcVer {
        patch3: B10,
        patch2: B10,
        patch1: B10,
        minor: B10,
        major: B24,
    }
    impl SrcVer {
        pub fn get_major(self) -> u32 { self.major() }
    }
    
    impl fmt::Display for SrcVer {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}.{}.{}.{}.{}", self.major(), self.minor(), self.patch1(), self.patch2(), self.patch3())
        }
    }
}

use srcver::SrcVer;

#[derive(BinRead, Debug)]
pub struct SEPMonitorBootArgs {
    //monitor related
    pub version: u32,   // Version of the monitor boot args
    pub virt_base: u32, // Virtual base address of the monitor
    pub phys_base: u32, // Physical base address of the monitor
    pub mem_size: u32,  // Size of the monitor's memory
    //kernel related
    pub args_off: u32, // offset to SEPKernBootArgs struct
    pub entry: u32,    // entry point/main function of the kernel (duplicated in SEPOS app info)
    /* headers say:
        pub kphys_base: u32,
        pub phys_slide: u32,
        pub virt_slide: u32
    but actual SEP firmware says: */ 
    pub uuid: [u8; 16]
}

#[derive(BinRead, Debug)]
pub struct SEPKernBootArgs {
    revision: u16,
    version: u16,
    virt_base: u32,
    phys_base: u32,
    mem_size: u32,
    top_of_kernel_data: u32,
    shm_base: u64,
    smh_size: u32,
    reserved: [u32; 3],
    sepos_crc32: u32,
    seprom_args_offset: u32,
    seprom_phys_offset: u32,
    entropy: [u64; 2],
    pub num_apps: u32,
    pub num_shlibs: u32,
    _unused: [u8; 232],
    /*
    on older SEPs (seen in iOS 10 A10) from 'entropy' until the end of "unused', there may be a string, stating:
    	Firmware magic string
		Without which, what are these bits?
		SEP denied.
    later firmwares do not have this string, and is instead just zeros.
    */
}

#[derive(BinRead, Debug, PartialEq, Eq)]
#[br(repr = u8)]
pub enum BootArgsType { //describes space between first fields and name
    A10     = 69, //major 18xx (e.g. iOS 14 A10)
    //is 69 because it uses the 64-bit struct anyways, first fields are different (just a random value, not the actual size)
    A9      = 24, //major 16xx
    A8      = 20, //major 12xx
  //A8Old   = 12, //major 8xx
    A10Old  = 12, //major 6xx
    OldFW   = 0,  //no version field, uses SEPAppOld struct
}

#[derive(BinRead, Debug)]
#[br(import(ver: u8))]
#[non_exhaustive]
pub struct SEPDataHDR64 {
    pub kernel_uuid: [u8; 16],      // The UUID of the kernel
    pub kernel_heap_size: u64,      // The size of the kernel's heap
    pub kernel_base_paddr: u64,     // The address of the kernel in the firmware
    pub kernel_max_paddr: u64,      // The maximum address of the kernel in the firmware
    pub app_images_base_paddr: u64, // The address of the apps in the firmware
    pub app_images_max_paddr: u64,  // The maximum address of the apps in the firmware
    pub paddr_max: u64,             // The size of the SEP firmware image
    pub tz0_min_size: u64,          // The minimum size of the TZ0 region
    pub tz1_min_size: u64,          // The minimum size of the TZ1 region
    pub ar_min_size: u64,           // The minimum size of the Anti Replay region
    //these do not exist in SEP < 1800
    #[br(if(ar_min_size != 0 || ver == 4, 0))]
    pub non_ar_min_size: u64,       // The minimum size of the non-Anti Replay region
    #[br(if(ar_min_size != 0 || ver == 4, 0))]
    pub shm_base: u64,              // The base address of the shared memory region
    #[br(if(ar_min_size != 0 || ver == 4, 0))]
    pub shm_size: u64,              // The size of the shared memory region
    //rootserver (SEPOS) info start
        pub init_base_paddr: u64,   // The physical address of SEPOS
        pub init_base_vaddr: u64,   // The virtual address of SEPOS
        pub init_vsize: u64,        // The initial virtual size of SEPOS
        pub init_ventry: u64,       // The entry/main function of SEPOS (from Mach-O start)
        pub stack_base_paddr: u64,  // The physical address of the SEPOS stack
        pub stack_base_vaddr: u64,  // The virtual address of the SEPOS stack
        pub stack_size: u64,        // The size of SEPOS's stack
        //these do not exist in iOS 13 SEP
        #[br(if(stack_size != 0 || ver == 4, 0))]
        pub mem_size: u64,          // The size of SEPOS's memory
        #[br(if(stack_size != 0 || ver == 4, 0))]
        pub antireplay_mem_size: u64, // The size of SEPOS's Anti Replay memory
        #[br(if(stack_size != 0 || ver == 4, 0))]
        pub heap_mem_size: u64,     // The size of SEPOS's heap
        #[br(if(ver == 4))]
        pub compact_ver_start: u32, // The start of the compact version (0xFFFF_FFFF if not versioned)
        #[br(if(ver == 4))]
        pub compact_ver_end: u32,   // The end of the compact version
        #[br(if(ver == 4))]
        _unk1: u64,
        #[br(if(ver == 4))]
        _unk2: u64,
        #[br(if(ver == 4))]
        _unk3: u64,
        pub init_name: [u8; 16],    // The name of the rootserver (usually SEPOS)
        pub init_uuid: [u8; 16],    // The UUID of the rootserver
        pub srcver: SrcVer,         // The source version of the rootserver
    //rootserver end
    pub crc32: u32, // CRC32 of all of the apps after SEPOS
    pub coredump_sup: u8, //actually bool but I don't want a panic in case it deserializes the wrong bytes
    pub pad: [u8; 3], //u32 alignment
    #[br(if(pad == [0x40, 0x04, 0x00], [0; 0x100]))]
    _unk4: [u8; 0x100], // 'set1', 'set2', ...
    pub n_apps: u32,      // The number of apps that follow
    pub n_shlibs: u32,    // The number of shared libraries that follow after the apps
}

#[derive(BinRead, Debug)]
#[br(import(ver: u8))]
/* right after the above, from offset 0x11c0 */
/* newest 32 bit SEPOS also uses this */
pub struct SEPApp64 {
    pub phys_text: u64, // The address of the app's Mach-O
    pub size_text: u64, // The size of the app's Mach-O (doesn't include rw segments, e.g. __DATA)
    pub phys_data: u64, // The address of the app's rw segments
    pub size_data: u64, // The size of the app's rw segments
    pub virt: u64,      // The virtual address of the app
    pub ventry: u64,    // The entry/main function of the app (from Mach-O start)
    pub stack_size: u64,// The size of the app's stack
    pub mem_size: u64,  // The size of the app's memory
    pub non_antireplay_mem_size: u64, // The size of the app's non-Anti Replay memory
    #[br(if(stack_size != 0 || ver == 4, 0))]
    pub heap_mem_size: u64, // The size of the app's heap memory
    #[br(if(ver == 4, 0))]
    _unk1: u64,
    #[br(if(ver == 4, 0))]
    _unk2: u64,
    #[br(if(ver == 4, 0))]
    _unk3: u64,
    #[br(if(ver == 4, 0))]
    _unk4: u64,
    pub compact_ver_start: u32, // The start of the compact version (0xFFFF_FFFF if not versioned)
    pub compact_ver_end: u32,   // The end of the compact version
    pub app_name: [u8; 16],     // The name of the app
    pub app_uuid: [u8; 16],     // The UUID of the app
    pub srcver: SrcVer,         // The source version of the app
}

/* unused struct
#[derive(BinRead, Debug)]
// SEPOS 16xx uses this, atleast for N71m SEP
pub struct SEPApp32 {
    pub phys_text: u32,
    pub virt_base: u32,
    pub size: u32,
    pub entry: u32,
    pub stack_size: u32,
    pub mem_size: u32,
    pub non_antireplay_mem_size: u32, //not present until A9
    pub heap_mem_size: u64, //not present until new A8
    pub compact_ver_start: u32,
    pub compact_ver_end: u32,
    pub app_name: [u8; 12],
    pub app_uuid: [u8; 16],
    pub srcver: SrcVer,
}
*/

#[derive(BinRead, Debug)]
/* first version of SEPOS bootargs */
pub struct SEPAppOld {
    pub phys: u64,  // The address of the app's Mach-O
    pub virt: u32,  // The virtual address of the app
    pub size: u32,  // The size of the app's Mach-O (includes __DATA)
    pub entry: u32, // The entry/main function of the app
    /* pub name: [u8; 12], */ // The name of the app
    /* char hash[16]; */      // Could also be UUID
}


//copied from Apple's loader.h
type VMProt = i32;
type CPUType = i32;
type CPUSubtype = i32;

#[derive(BinRead, Debug)]
pub struct MachHeader {
    pub magic: u32,
    pub cputype: CPUType,
    pub cpusubtype: CPUSubtype,
    pub filetype: u32,
    pub ncmds: u32,
    pub sizeofcmds: u32,
    pub flags: u32,
}

#[binrw]
#[derive(Debug)]
pub struct Segment {
    pub segname: [u8; 16],
    pub vmaddr: u32,
    pub vmsize: u32,
    pub fileoff: u32,
    pub filesize: u32,
    pub maxprot: VMProt,
    pub initprot: VMProt,
    pub flags: u32
}

#[binrw]
#[derive(Debug)]
pub struct Segment64 {
    pub segname: [u8; 16],
    pub vmaddr: u64,
    pub vmsize: u64,
    pub fileoff: u64,
    pub filesize: u64,
    pub maxprot: VMProt,
    pub initprot: VMProt,
    pub flags: u32,
}

#[binrw]
#[derive(Debug, Default)]
pub struct SymTab {
    pub symoff: u32,
    pub nsyms: u32,
    pub stroff: u32,
    pub strsize: u32,
}

#[binrw]
#[derive(Debug, Default)]
pub struct DySymTab {
    pub ilocalsym: u32,
    pub nlocalsym: u32,
    pub iextdefsym: u32,
    pub nextdefsym: u32,
    pub iundefsym: u32,
    pub nundefsym: u32,
    pub tocoff: u32,
    pub ntoc: u32,
    pub modtaboff: u32,
    pub nmodtab: u32,
    pub extrefsymoff: u32,
    pub nextrefsyms: u32,
    pub indirectsymoff: u32,
    pub nindirectsyms: u32,
    pub extreloff: u32,
    pub nextrel: u32,
    pub locreloff: u32,
    pub nlocrel: u32,
}

#[derive(BinRead)]
pub struct SrcVerCmd {
    pub cmd: u32,	        /* LC_SOURCE_VERSION */
    pub cmdsize: u32,	    /* 16 */
    pub version: SrcVer,	/* A.B.C.D.E packed as a24.b10.c10.d10.e10 */
}

//type of command in cmd field
#[derive(PartialEq, Eq, Debug, BinRead)]
pub enum Cmd {
    #[br(magic = 0x1u32)] Segment,
    #[br(magic = 0x19u32)] Segment64,
    #[br(magic = 0x2u32)] SymTab,
    #[br(magic = 0xBu32)] DySymTab,
    #[br(magic = 0x2Au32)] SourceVersion,
    Unknown = 0xFFFF
}

#[derive(BinRead, Debug)]
pub struct LoadCommand {
    pub cmd: Cmd,
    pub cmdsize: u32
}

#[derive(Debug, PartialEq, Eq)]
pub struct SEPinfo {
    pub sep_app_pos: usize,
    pub sepapp_size: usize,
    pub sepapps: Option<usize>,
    pub shlibs: Option<usize>
}


pub static SEG_DATA:     [u8; 16] = *b"__DATA\0\0\0\0\0\0\0\0\0\0";
pub static SEG_PAGEZERO: [u8; 16] = *b"__PAGEZERO\0\0\0\0\0\0";
pub static SEG_LINKEDIT: [u8; 16] = *b"__LINKEDIT\0\0\0\0\0\0";

//pub static LEGION_32_SIZE:  usize = 22;
//pub static LEGION_64_SIZE:  usize = 22;
pub static SEPHDR_SIZE:       usize = 224;
pub static SEPAPP_64_SIZE:    usize = 128;
pub static SEPAPP_SIZE:       usize = 32;
pub static MACHHEADER_SIZE:   usize = 28;
pub static LOADCOMMAND_SIZE:  usize = 8;
pub static KRNLBOOTARGS_SIZE: usize = 312;
//pub static SEGMENT_SIZE: usize = 64;
//pub static SEGMENT64_SIZE: usize = 80;

impl MachHeader {
    pub const fn is_macho(&self) -> bool { self.magic & 0xffff_fffe == 0xfeed_face } //bitwise AND with 0x0 ignores 64 bit
    pub const fn is64(&self) -> bool { self.magic & 0x1 == 1 } // would mean 0xfeed_facf
}
