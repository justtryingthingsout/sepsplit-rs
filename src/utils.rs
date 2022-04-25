use std::{fmt};
use serde::{Serialize, Deserialize}; //de::DeserializeOwned
use serde_big_array::BigArray;
use binrw::{BinRead};

#[macro_export]
macro_rules! range_size {
    ($start: expr, $size:expr) => {
        $start..$start+$size
    }
}

#[macro_export]
macro_rules! cast_struct {
    ($t: ty, $arr: expr) => {
        bincode::deserialize::<$t>($arr).unwrap_or_else(|_| panic!("Unable to deserialize to {}", stringify!($t)))
    }
}

//structs
// #[derive(Serialize, Deserialize)]
// pub struct Legion64 {
//     pub subversion: u32, //0x3
//     pub legionstr: [u8; 16],
//     pub structoff: u16
//     reserved: [u8; 2]
// }
// 
#[derive(Serialize, Deserialize)]
pub struct Legion32 {
    pub subversion: u32, //0x1
    pub off: u32, //0x800
    pub legionstr: [u8; 16]
}

#[derive(BinRead, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SrcVer(u64);

#[derive(Serialize, Deserialize, Debug)]
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
    but actual says: */ 
    pub uuid: [u8; 16]
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SEPKernBootArgs {
    version: u16,
    revision: u16,
    zero1: [u32; 3],
    pub ver: u32, //possible version field
    zero2: [u32; 6],
    crc32: u32,
    zero3: u64,
    #[serde(with = "BigArray")]
    unused: [u8; 256]
    /*
    unused may contain a string on older SEPs, stating:
    	Firmware magic string
		Without which, what are these bits?
		SEP denied.
    */
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum BootArgsType { //describes space between first fields and name
    A10     = 69, //major 18xx (e.g. iOS 14 A10)
    //is 69 because it uses the 64-bit struct anyways, first fields are different
    A9      = 24, //major 16xx
    A8      = 20, //major 12xx
    //A8Old   = 12, //major 8xx
    A10Old  = 12, //major 6xx
    OldFW   = 0,  //no version field, uses SEPApp
}

#[derive(BinRead, Debug)]
pub struct SEPDataHDR64 {
    pub kernel_uuid: [u8; 16],
    pub kernel_heap_size: u64,
    pub kernel_base_paddr: u64,
    pub kernel_max_paddr: u64,
    pub app_images_base_paddr: u64,
    pub app_images_max_paddr: u64,
    pub paddr_max: u64, /* size of SEP firmware image */
    pub tz_min_size: u64,
    pub shm_base: u64,
    pub shm_size: u64,
    //these do not exist in SEP < 1800
    #[br(if(shm_size != 0, 0))]
    _unknown1: u64,
    #[br(if(shm_size != 0, [0, 0]))]
    _unknown2: [u64; 2],
    //rootserver start
        pub init_base_paddr: u64,
        pub init_base_vaddr: u64,
        pub init_vsize: u64,
        pub init_ventry: u64,
        pub stack_base_paddr: u64,
        pub stack_base_vaddr: u64,
        pub stack_size: u64,
        //start
        pub mem_size: u64,
        pub antireplay_mem_size: u64,
        pub heap_mem_size: u64,
        //exit
        pub init_name: [u8; 16],
        pub init_uuid: [u8; 16],
        pub srcver: SrcVer,
    //rootserver end
    pub crc32: u32,
    pub coredump_sup: u8, //bool but i don't want invalid values
    _pad: [u8; 3],
    pub n_apps: u64,
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub heap_mem_size: u64,
    pub compact_ver_start: u32,
    pub compact_ver_end: u32,
    pub app_name: [u8; 16],
    pub app_uuid: [u8; 16],
    pub srcver: SrcVer,
}

/*
#[derive(Serialize, Deserialize, Debug)]
/* SEPOS 16xx uses this, atleast for N71m SEP */
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

#[derive(Serialize, Deserialize, Debug)]
/* first version of SEPOS bootargs */
pub struct SEPAppOld {
    pub phys: u64,
    pub virt: u32,
    pub size: u32,
    pub entry: u32,
    /* pub name: [u8; 12], */
    /* char hash[16]; //could also be UUID */
}

type VMProt = i32;
type CPUType = i32;
type CPUSubtype = i32;

#[derive(Serialize, Deserialize, Debug)]
pub struct MachHeader {
    pub magic: u32,
    pub cputype: CPUType,
    pub cpusubtype: CPUSubtype,
    pub filetype: u32,
    pub ncmds: u32,
    pub sizeofcmds: u32,
    pub flags: u32,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct SymTab {
    pub symoff: u32,
    pub nsyms: u32,
    pub stroff: u32,
    pub strsize: u32,
}

#[derive(Serialize, Deserialize)]
pub struct SrcVerCmd {
    pub cmd: u32,	        /* LC_SOURCE_VERSION */
    pub cmdsize: u32,	    /* 16 */
    pub version: SrcVer,	/* A.B.C.D.E packed as a24.b10.c10.d10.e10 */
}

#[repr(u32)]
#[derive(PartialEq)]
pub enum CMD {
    Segment = 0x1,
    Segment64 = 0x19,
    SymTab = 0x2,
    SourceVersion = 0x2A,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoadCommand {
    pub cmd: u32,
    pub cmdsize: u32
}

#[derive(Debug, PartialEq)]
pub struct SEPinfo {
    pub sep_app_pos: usize,
    pub sepapp_size: usize,
}


pub static SEG_DATA:     &[u8; 16] = b"__DATA\0\0\0\0\0\0\0\0\0\0";
pub static SEG_PAGEZERO: &[u8; 16] = b"__PAGEZERO\0\0\0\0\0\0";
pub static SEG_LINKEDIT: &[u8; 16] = b"__LINKEDIT\0\0\0\0\0\0";

//pub static LEGION_32_SIZE:  usize = 22;
//pub static LEGION_64_SIZE:  usize = 22;
pub static SEPHDR_SIZE:     usize = 224;
pub static SEPAPP_64_SIZE:  usize = 128;
pub static SEPAPP_SIZE:     usize = 32;
pub static MACHHEADER_SIZE: usize = 28;
pub static LOADCOMMAND_SIZE: usize = 8;
pub static KRNLBOOTARGS_SIZE:  usize = 312;
//pub static SEGMENT_SIZE: usize = 64;
//pub static SEGMENT64_SIZE: usize = 80;
//machhead, seg, seg64, symt

impl MachHeader {
    pub fn is_macho(&self) -> bool { self.magic & 0xffff_fffe == 0xfeed_face } //AND with 0x0 ignores 64 bit
    pub fn is64(&self) -> bool { self.magic & 0x1 == 1 } // would mean 0xfeed_facf
}

impl fmt::Display for SrcVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num = self.0 as u64;
        let major = num >> 40;
        let minor = num >> 30 & 0x3ff;
        let patch1 = num >> 20 & 0x3ff;
        let patch2 = num >> 10 & 0x3ff;
        let patch3 = num & 0x3ff;
        write!(f, "{}.{}.{}.{}.{}", major, minor, patch1, patch2, patch3)
    }
}

impl SrcVer {
    pub fn get_major(&self) -> u64 {
        &self.0 >> 40
    }
}

impl TryFrom<u32> for CMD {
    type Error = ();

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        if v > 0x2B { panic!("this is not a cmd, value was {:x}", v) }
        match v {
            x if x == CMD::Segment as u32 => Ok(CMD::Segment),
            x if x == CMD::Segment64 as u32 => Ok(CMD::Segment64),
            x if x == CMD::SymTab as u32 => Ok(CMD::SymTab),
            x if x == CMD::SourceVersion as u32 => Ok(CMD::SourceVersion),
            _ => Err(()),
        }
    }
}