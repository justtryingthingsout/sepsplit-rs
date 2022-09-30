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
    pub version: u32,
    pub virt_base: u32,
    pub phys_base: u32,
    pub mem_size: u32,
    //kernel related
    pub args_off: u32, // offset to SEPKernBootArgs struct
    pub entry: u32,
    /* headers say:
        pub kphys_base: u32,
        pub phys_slide: u32,
        pub virt_slide: u32
    but actual SEP firmware says: */ 
    pub uuid: [u8; 16]
}

#[derive(BinRead, Debug)]
pub struct SEPKernBootArgs {
    _revision: u16,
    _version: u16,
    _virt_base: u32,
    _phys_base: u32,
    _mem_size: u32,
    _top_of_kernel_data: u32,
    _shm_base: u64,
    _smh_size: u32,
    _reserved: [u32; 3],
    _sepos_crc32: u32,
    _seprom_args_offset: u32,
    _seprom_phys_offset: u32,
    _entropy: [u64; 2],
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
pub struct SEPDataHDR64 {
    pub kernel_uuid: [u8; 16],
    pub kernel_heap_size: u64,
    pub kernel_base_paddr: u64,
    pub kernel_max_paddr: u64,
    pub app_images_base_paddr: u64,
    pub app_images_max_paddr: u64,
    pub paddr_max: u64, /* size of SEP firmware image */
    pub tz0_min_size: u64,
    pub tz1_min_size: u64,
    pub ar_min_size: u64,
    //these do not exist in SEP < 1800
    #[br(if(ar_min_size != 0 || ver == 4, 0))]
    pub non_ar_min_size: u64,
    #[br(if(ar_min_size != 0 || ver == 4, 0))]
    pub shm_base: u64,
    #[br(if(ar_min_size != 0 || ver == 4, 0))]
    pub shm_size: u64,
    //rootserver start
        pub init_base_paddr: u64,
        pub init_base_vaddr: u64,
        pub init_vsize: u64,
        pub init_ventry: u64,
        pub stack_base_paddr: u64,
        pub stack_base_vaddr: u64,
        pub stack_size: u64,
        //these do not exist in iOS 13 SEP
        #[br(if(stack_size != 0 || ver == 4, 0))]
        pub mem_size: u64,
        #[br(if(stack_size != 0 || ver == 4, 0))]
        pub antireplay_mem_size: u64,
        #[br(if(stack_size != 0 || ver == 4, 0))]
        pub heap_mem_size: u64,
        #[br(if(ver == 4))]
        pub compact_ver_start: u32,
        #[br(if(ver == 4))]
        pub compact_ver_end: u32,
        #[br(if(ver == 4))]
        _unk1: u64,
        #[br(if(ver == 4))]
        _unk2: u64,
        #[br(if(ver == 4))]
        _unk3: u64,
        pub init_name: [u8; 16],
        pub init_uuid: [u8; 16],
        pub srcver: SrcVer,
    //rootserver end
    pub crc32: u32,
    pub coredump_sup: u8, //actually bool but I don't want a panic in case it deserializes the wrong bytes
    _pad: [u8; 3], //u32 alignment
    pub n_apps: u32,
    pub n_shlibs: u32,
}

#[derive(BinRead, Debug)]
#[br(import(ver: u8))]
/* right after the above, from offset 0x11c0 */
/* newest 32 bit SEPOS also uses this */
pub struct SEPApp64 {
    pub phys_text: u64,
    pub size_text: u64,
    pub phys_data: u64,
    pub size_data: u64,
    pub virt: u64,
    pub ventry: u64,
    pub stack_size: u64,
    pub mem_size: u64,
    pub non_antireplay_mem_size: u64,
    #[br(if(stack_size != 0 || ver == 4, 0))]
    pub heap_mem_size: u64,
    #[br(if(ver == 4, 0))]
    _unk1: u64,
    #[br(if(ver == 4, 0))]
    _unk2: u64,
    #[br(if(ver == 4, 0))]
    _unk3: u64,
    #[br(if(ver == 4, 0))]
    _unk4: u64,
    pub compact_ver_start: u32,
    pub compact_ver_end: u32,
    pub app_name: [u8; 16],
    pub app_uuid: [u8; 16],
    pub srcver: SrcVer,
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
    pub phys: u64,
    pub virt: u32,
    pub size: u32,
    pub entry: u32,
    /* pub name: [u8; 12], */
    /* char hash[16]; //could also be UUID */
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
#[derive(Debug)]
pub struct SymTab {
    pub symoff: u32,
    pub nsyms: u32,
    pub stroff: u32,
    pub strsize: u32,
}

#[derive(BinRead)]
pub struct SrcVerCmd {
    pub cmd: u32,	        /* LC_SOURCE_VERSION */
    pub cmdsize: u32,	    /* 16 */
    pub version: SrcVer,	/* A.B.C.D.E packed as a24.b10.c10.d10.e10 */
}

//type of command in cmd field
#[repr(u32)]
#[derive(PartialEq, Eq)]
pub enum CMD {
    Segment = 0x1,
    Segment64 = 0x19,
    SymTab = 0x2,
    SourceVersion = 0x2A,
}

#[derive(BinRead, Debug)]
pub struct LoadCommand {
    pub cmd: u32,
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



impl TryFrom<u32> for CMD {
    type Error = (); //either panics or isn't a real error, so () is fine

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        assert!(v & !(1 << 31) & !(1 << 27) <= 0x100, "this is not a cmd, value was {:#x}", v);
        match v { // https://stackoverflow.com/a/57578431
            x if x == Self::Segment       as u32 => Ok(Self::Segment),
            x if x == Self::Segment64     as u32 => Ok(Self::Segment64),
            x if x == Self::SymTab        as u32 => Ok(Self::SymTab),
            x if x == Self::SourceVersion as u32 => Ok(Self::SourceVersion),
            _ => Err(()),
        }
    }
}