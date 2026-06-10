#![forbid(unsafe_code)]
#![allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]

//#[cfg(DEBUG_STATE_ENABLED)] 
//use std::print as _LZVN_DEBUG_DUMP; 
//#[cfg(not(DEBUG_STATE_ENABLED))] 
macro_rules! _LZVN_DEBUG_DUMP {
	($($e:expr),*) => {}
}

macro_rules! read_u64 {
	($buf: expr) => {
		u64::from_le_bytes($buf[0..8].try_into().unwrap())
	}
}

macro_rules! write_u64 {
	($res: expr, $num: expr) => {
		$res[0..8].copy_from_slice(&$num.to_le_bytes())
	}
}

const LZVN_0: u8 = 0;
const LZVN_1: u8 = 1;
const LZVN_2: u8 = 2;
const LZVN_3: u8 = 3;
const LZVN_4: u8 = 4;
const LZVN_5: u8 = 5;
const LZVN_6: u8 = 6;
const LZVN_7: u8 = 7;
const LZVN_8: u8 = 8;
const LZVN_9: u8 = 9;
const LZVN_10: u8 = 10;
const LZVN_11: u8 = 11;

const JUMP_CASE_TABLE: u8 = 127;

//==============================================================================

const CASE_TABLE: [ u16; 256 ] = [
		1,  1,  1,  1,    1,  1,  2,  3,    1,  1,  1,  1,    1,  1,  4,  3,
		1,  1,  1,  1,    1,  1,  4,  3,    1,  1,  1,  1,    1,  1,  5,  3,
		1,  1,  1,  1,    1,  1,  5,  3,    1,  1,  1,  1,    1,  1,  5,  3,
		1,  1,  1,  1,    1,  1,  5,  3,    1,  1,  1,  1,    1,  1,  5,  3,
		1,  1,  1,  1,    1,  1,  0,  3,    1,  1,  1,  1,    1,  1,  0,  3,
		1,  1,  1,  1,    1,  1,  0,  3,    1,  1,  1,  1,    1,  1,  0,  3,
		1,  1,  1,  1,    1,  1,  0,  3,    1,  1,  1,  1,    1,  1,  0,  3,
		5,  5,  5,  5,    5,  5,  5,  5,    5,  5,  5,  5,    5,  5,  5,  5,
		1,  1,  1,  1,    1,  1,  0,  3,    1,  1,  1,  1,    1,  1,  0,  3,
		1,  1,  1,  1,    1,  1,  0,  3,    1,  1,  1,  1,    1,  1,  0,  3,
		6,  6,  6,  6,    6,  6,  6,  6,    6,  6,  6,  6,    6,  6,  6,  6,
		6,  6,  6,  6,    6,  6,  6,  6,    6,  6,  6,  6,    6,  6,  6,  6,
		1,  1,  1,  1,    1,  1,  0,  3,    1,  1,  1,  1,    1,  1,  0,  3,
		5,  5,  5,  5,    5,  5,  5,  5,    5,  5,  5,  5,    5,  5,  5,  5,
		7,  8,  8,  8,    8,  8,  8,  8,    8,  8,  8,  8,    8,  8,  8,  8,
		9, 10, 10, 10,   10, 10, 10, 10,   10, 10, 10, 10,   10, 10, 10, 10
];

struct LZVNState<'a> {
    decompressed_size: usize,
    compressed_size: usize,
    decomp_buffer: &'a mut [u8],
    length: usize,
	comp_buffer: &'a [u8],
    comp_buffer_off: usize,
    comp_buffer_pointer: u64,	// use p(ointer)?
	case_table_index: u64,
	byte_count: u64,
	current_length: u64,																// xor	%r12,%r12
	negative_offset: u64,
	address: usize,
    jmp_to: u8
}

#[derive(PartialEq, Clone)]
enum LZVNRes {
    Exit(usize),
    FallThrough,
    Jump
}

impl LZVNRes {
	fn map_fall<F: FnOnce() -> Self>(self, f: F) -> Self {
		if FallThrough == self {
			f()
		} else {
			self
		}
	}
}

use self::LZVNRes::{Exit, FallThrough, Jump};

const fn do_case_table_0(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("caseTable[0]\n");

    state.case_table_index >>= 6;												// shr	$0x6,%r9
    state.comp_buffer_off += state.case_table_index as usize + 1;				// lea	0x1(%rdx,%r9,1),%rdx

    if state.comp_buffer_off > state.compressed_size							// cmp	%rcx,%rdx
    {
        _LZVN_DEBUG_DUMP!("FAIL C0");
		return Exit(0);															// ja	Llzvn_exit
    }

    state.byte_count = 56;														// mov	$0x38,%r10
    state.byte_count &= state.comp_buffer_pointer;								// and	%r8,%r10
    state.comp_buffer_pointer >>= 8;											// shr	$0x8,%r8
    state.byte_count >>= 3;														// shr	$0x3,%r10
    state.byte_count += 3;														// add	$0x3,%r10

    state.jmp_to = LZVN_10;														// jmp	Llzvn_l10
    Jump
}

const fn do_case_table_1(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("caseTable[1]\n");

	state.case_table_index >>= 6;												// shr	$0x6,%r9
	state.comp_buffer_off += state.case_table_index as usize + 2;				// lea	0x2(%rdx,%r9,1),%rdx

	if state.comp_buffer_off > state.compressed_size							// cmp	%rcx,%rdx
	{
		_LZVN_DEBUG_DUMP!("FAIL C1");
		return Exit(0);															// ja	Llzvn_exit
	}
	
	state.negative_offset = state.comp_buffer_pointer;							// mov	%r8,%r12
	state.negative_offset = state.negative_offset.swap_bytes();					// bswap	%r12
	state.byte_count = state.negative_offset;									// mov	%r12,%r10
	state.negative_offset <<= 5;												// shl	$0x5,%r12
	state.byte_count <<= 2;														// shl	$0x2,%r10
	state.negative_offset >>= 53;												// shr	$0x35,%r12
	state.byte_count >>= 61;													// shr	$0x3d,%r10
	state.comp_buffer_pointer >>= 16;											// shr	$0x10,%r8
	state.byte_count += 3;														// add	$0x3,%r10

	state.jmp_to = LZVN_10;														// jmp	Llzvn_l10
    Jump
}

const fn do_case_table_2(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("caseTable[2]\n");

	Exit(state.length)
}

const fn do_case_table_3(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("caseTable[3]\n");

	state.case_table_index >>= 6;												// shr	$0x6,%r9
	state.comp_buffer_off += 
		state.case_table_index as usize + 3;									// lea	0x3(%rdx,%r9,1),%rdx

	if state.comp_buffer_off > state.compressed_size							// cmp	%rcx,%rdx
	{
		_LZVN_DEBUG_DUMP!("FAIL C3");
		return Exit(0);															// ja	Llzvn_exit
	}
	
	state.byte_count = 56;														// mov	$0x38,%r10
	state.negative_offset = 0xffff;												// mov	$0xffff,%r12
	state.byte_count &= state.comp_buffer_pointer;								// and	%r8,%r10
	state.comp_buffer_pointer >>= 8;											// shr	$0x8,%r8
	state.byte_count >>= 3;														// shr	$0x3,%r10
	state.negative_offset &= state.comp_buffer_pointer;							// and	%r8,%r12
	state.comp_buffer_pointer >>= 16;											// shr	$0x10,%r8
	state.byte_count += 3;														// add	$0x3,%r10
	
	state.jmp_to = LZVN_10;														// jmp	Llzvn_l10
    Jump
}

fn do_case_table_4(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("caseTable[4]\n");

	state.comp_buffer_off += 1;													// add	$0x1,%rdx

	if state.comp_buffer_off > state.compressed_size							// cmp	%rcx,%rdx
	{								
		_LZVN_DEBUG_DUMP!("FAIL C4");
		return Exit(0);															// ja	Llzvn_exit
	}
	
	state.comp_buffer_pointer = 
		read_u64!(state.comp_buffer[state.comp_buffer_off..]);					// mov	(%rdx),%r8
	state.case_table_index = state.comp_buffer_pointer & 255;					// movzbq (%rdx),%r9
	
	state.jmp_to = JUMP_CASE_TABLE;												// continue;
    Jump
}

const fn do_case_table_5(_state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("caseTable[5]\n");

	Exit(0)																		// Llzvn_table5;
}

const fn do_case_table_6(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("caseTable[6]\n");

	state.case_table_index >>= 3;												// shr	$0x3,%r9
	state.case_table_index &= 3;												// and	$0x3,%r9
	state.comp_buffer_off += 
		state.case_table_index as usize + 3;									// lea	0x3(%rdx,%r9,1),%rdx

	if state.comp_buffer_off > state.compressed_size							// cmp	%rcx,%rdx
	{
		_LZVN_DEBUG_DUMP!("FAIL C6");
		return Exit(0);															// ja	Llzvn_exit
	}
	
	state.byte_count = state.comp_buffer_pointer;								// mov	%r8,%r10
	state.byte_count &= 0x307;													// and	$0x307,%r10
	state.comp_buffer_pointer >>= 10;											// shr	$0xa,%r8
	state.negative_offset = state.byte_count & 255;								// movzbq %r10b,%r12
	state.byte_count >>= 8;														// shr	$0x8,%r10
	state.negative_offset <<= 2;												// shl	$0x2,%r12
	state.byte_count |= state.negative_offset;									// or	%r12,%r10
	state.negative_offset = 0x3fff;												// mov	$0x3fff,%r12
	state.byte_count += 3;														// add	$0x3,%r10
	state.negative_offset &= state.comp_buffer_pointer;							// and	%r8,%r12
	state.comp_buffer_pointer >>= 14;											// shr	$0xe,%r8

	state.jmp_to = LZVN_10;														// jmp	Llzvn_l10
    Jump
}

const fn do_case_table_7(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("caseTable[7]\n");

	state.comp_buffer_pointer >>= 8;											// shr	$0x8,%r8
	state.comp_buffer_pointer &= 255;											// and	$0xff,%r8
	state.comp_buffer_pointer += 16;											// add	$0x10,%r8
	state.comp_buffer_off += 
		(state.comp_buffer_pointer + 2) as usize;								// lea	0x2(%rdx,%r8,1),%rdx

	state.jmp_to = LZVN_0;														// jmp	Llzvn_l0
    Jump
}

const fn do_case_table_8(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("caseTable[8]\n");

	state.comp_buffer_pointer &= 15;											// and	$0xf,%r8
	state.comp_buffer_off += 									
		(state.comp_buffer_pointer + 1) as usize;								// lea	0x1(%rdx,%r8,1),%rdx

	state.jmp_to = LZVN_0;														// jmp	Llzvn_l0
    Jump
}

const fn do_case_table_9(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("caseTable[9]\n");

	state.comp_buffer_off += 2;													// add	$0x2,%rdx
	
	if state.comp_buffer_off > state.compressed_size							// cmp	%rcx,%rdx
	{
		_LZVN_DEBUG_DUMP!("FAIL C9");
		return Exit(0);															// ja	Llzvn_exit
	}

	// Up most significant byte (count) by 16 (0x10/16 - 0x10f/271).
	state.byte_count = state.comp_buffer_pointer;								// mov	%r8,%r10
	state.byte_count >>= 8;														// shr	$0x8,%r10
	state.byte_count &= 255;													// and	$0xff,%r10
	state.byte_count += 16;														// add	$0x10,%r10

	state.jmp_to = LZVN_11;														// jmp	Llzvn_l11
    Jump
}

const fn do_case_table_10(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("caseTable[10]\n");

	state.comp_buffer_off += 1;													// add	$0x1,%rdx
	
	if state.comp_buffer_off > state.compressed_size							// cmp	%rcx,%rdx
	{
		_LZVN_DEBUG_DUMP!("FAIL C10");
		return Exit(0);															// ja	Llzvn_exit
	}
	
	state.byte_count = state.comp_buffer_pointer;								// mov	%r8,%r10
	state.byte_count &= 15;														// and	$0xf,%r10
	
	state.jmp_to = LZVN_11;														// jmp	Llzvn_l11
    Jump
}

const fn do_lzvn_0(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("jmpTable(0)\n");

	if state.comp_buffer_off > state.compressed_size							// cmp	%rcx,%rdx
	{
		_LZVN_DEBUG_DUMP!("FAIL L0");
		return Exit(0);															// ja	Llzvn_exit
	}
	
	state.current_length = state.length as u64 + state.comp_buffer_pointer;		// lea	(%rax,%r8,1),%r11
	state.comp_buffer_pointer = 0_u64.wrapping_sub(state.comp_buffer_pointer);	// neg	%r8
	
	if state.current_length > state.decompressed_size as u64					// cmp	%rsi,%r11
	{
		state.jmp_to = LZVN_2;													// ja	Llzvn_l2
		return Jump;
	}

	//state.current_length += state.decomp_buffer.as_ptr() as u64;				// lea	(%rdi,%r11,1),%r11
    FallThrough
}

fn do_lzvn_1(state: &mut LZVNState) -> LZVNRes {
    loop																		// Llzvn_l1:
	{
		_LZVN_DEBUG_DUMP!("jmpTable(1)\n");

//		caseTableIndex = *(uint64_t *)((uint64_t)compBuffer + compBufferPointer);

		state.address = (state.comp_buffer_off as u64)
			.wrapping_add(state.comp_buffer_pointer) as usize;					// mov	(%rdx,%r8,1),%r9
		state.case_table_index = read_u64!(state.comp_buffer[state.address..]);

//		*(uint64_t *)((uint64_t)currentLength + compBufferPointer) = caseTableIndex;
// or:
//		memcpy((void *)currentLength + compBufferPointer, &caseTableIndex, 8);
// or:
		state.address = state.current_length
			.wrapping_add(state.comp_buffer_pointer) as usize;					// mov	%r9,(%r11,%r8,1)
		write_u64!(state.decomp_buffer[state.address..], 
			state.case_table_index);
		state.comp_buffer_pointer = 
			state.comp_buffer_pointer.wrapping_add(8);							// add	$0x8,%r8
        if (u64::MAX - (state.comp_buffer_pointer.wrapping_sub(8))) < 8 {
            break;
        }
	}																			// jae	Llzvn_l1

	state.length = state.current_length as usize;								// mov	%r11,%rax
	//state.length -= state.decomp_buffer.as_ptr() as usize;					// sub	%rdi,%rax
	
	state.comp_buffer_pointer = 
		read_u64!(state.comp_buffer[state.comp_buffer_off..]);					// mov	(%rdx),%r8
	state.case_table_index = state.comp_buffer_pointer & 255;					// movzbq (%rdx),%r9

	state.jmp_to = JUMP_CASE_TABLE;
    Jump
}

const fn do_lzvn_2(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("jmpTable(2)\n");

	state.current_length = state.decompressed_size as u64 + 8;					// lea	0x8(%rsi),%r11
    FallThrough
}

fn do_lzvn_3(state: &mut LZVNState) -> LZVNRes {
    loop																		// Llzvn_l3: (block copy of bytes)
	{
		_LZVN_DEBUG_DUMP!("jmpTable(3)\n");

		state.address = (state.comp_buffer_off as u64)
			.wrapping_add(state.comp_buffer_pointer) as usize;					// movzbq (%rdx,%r8,1),%r9
		state.case_table_index = 
			read_u64!(state.comp_buffer[state.address..]) & 255;
		state.decomp_buffer[state.length] = 
			state.case_table_index.to_le_bytes()[0];
		state.length += 1;														// add	$0x1,%rax
		
		if state.current_length == state.length as u64							// cmp	%rax,%r11
		{
			return Exit(state.length);											// je	Llzvn_exit2
		}
		
		state.comp_buffer_pointer += 1;											// add	$0x1,%r8
        if state.comp_buffer_pointer as i64 == 0 {
            break;
        }
	}																			// jne	Llzvn_l3
	
	state.comp_buffer_pointer = 
		read_u64!(state.comp_buffer[state.comp_buffer_off..]);					// mov	(%rdx),%r8
	state.case_table_index = state.comp_buffer_pointer & 255;					// movzbq	(%rdx),%r9

	state.jmp_to = JUMP_CASE_TABLE;
    Jump
}

const fn do_lzvn_4(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("jmpTable(4)\n");

    state.current_length = state.decompressed_size as u64 + 8;					// lea	0x8(%rsi),%r11
    FallThrough
}

fn do_lzvn_5(state: &mut LZVNState) -> LZVNRes {
    loop																		// Llzvn_l5: (block copy of qwords)
	{
		_LZVN_DEBUG_DUMP!("jmpTable(5)\n");

		state.address = state.comp_buffer_pointer as usize;						// mov	(%rdi,%r8,1),%r9
		state.case_table_index = 
			read_u64!(state.decomp_buffer[state.address..]);

		state.comp_buffer_pointer += 8;											// add	$0x8,%r8
		write_u64!(state.decomp_buffer[state.length..], 
			state.case_table_index);											// mov	%r9,(%rdi,%rax,1)
		state.length += 8;														// add	$0x8,%rax
		state.byte_count = state.byte_count.wrapping_sub(8);					// sub	$0x8,%r10
		if state.byte_count.wrapping_add(8) <= 8 {
            break;
        }
	}																			// ja	Llzvn_l5

	state.length = (state.length as u64)
		.wrapping_add(state.byte_count) as usize;								// add	%r10,%rax
	state.comp_buffer_pointer = 
		read_u64!(state.comp_buffer[state.comp_buffer_off..]);					// mov	(%rdx),%r8
	state.case_table_index = state.comp_buffer_pointer & 255;					// movzbq	(%rdx),%r9

	state.jmp_to = JUMP_CASE_TABLE;
    Jump
}

fn do_lzvn_6(state: &mut LZVNState) -> LZVNRes {
    loop
	{
		_LZVN_DEBUG_DUMP!("jmpTable(6)\n");

		state.decomp_buffer[state.length] = 
			state.case_table_index.to_le_bytes()[0];							// mov	%r8b,(%rdi,%rax,1)
		state.length += 1;														// add	$0x1,%rax
			
		if state.length as u64 == state.current_length							// cmp	%rax,%r11
		{
			return Exit(state.length);											// je	Llzvn_exit2
		}
			
		state.comp_buffer_pointer >>= 8;										// shr	$0x8,%r8
		state.case_table_index = state.case_table_index.wrapping_sub(1);		// sub	$0x1,%r9
		if state.case_table_index == 1 {
            break;
        }
	}																			// jne	Llzvn_l6
    FallThrough
}

const fn do_lzvn_7(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("jmpTable(7)\n");

	state.comp_buffer_pointer = state.length as u64;							// mov	%rax,%r8
	state.comp_buffer_pointer -= state.negative_offset;							// sub	%r12,%r8

	if state.comp_buffer_pointer < state.negative_offset						// jb	Llzvn_exit
	{
		_LZVN_DEBUG_DUMP!("FAIL L7");
		return Exit(0);
	}

	state.jmp_to = LZVN_4;
	Jump
}

const fn do_lzvn_8(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("jmpTable(8)\n");  
    if state.case_table_index == 0												// test	%r9,%r9
    {
    	state.jmp_to = LZVN_7;													// jmpq	*(%rbx,%r9,8)
    	return Jump;
    }   
    state.current_length = state.decompressed_size as u64 + 8;					// lea	0x8(%rsi),%r11
    FallThrough
}

fn do_lzvn_9(state: &mut LZVNState) -> LZVNRes {
    loop																		// Llzvn_l9: (block copy of bytes)
	{
		_LZVN_DEBUG_DUMP!("jmpTable(9)\n");

		state.address = state.comp_buffer_pointer as usize;						// movzbq (%rdi,%r8,1),%r9
		state.case_table_index = u64::from(state.decomp_buffer[state.address]);

		state.comp_buffer_pointer += 1;											// add	$0x1,%r8
		state.decomp_buffer[state.length] = 
			state.case_table_index.to_le_bytes()[0];							// mov	%r9,(%rdi,%rax,1)
		state.length += 1;														// add	$0x1,%rax
		
		if state.length as u64 == state.current_length							// cmp	%rax,%r11
		{
			return Exit(state.length);											// je	Llzvn_exit2
		}

		state.byte_count -= 1;													// sub	$0x1,%r10
		if state.byte_count == 0 {
            break;
        }
	}																			// jne	Llzvn_l9

	state.comp_buffer_pointer = 
		read_u64!(state.comp_buffer[state.comp_buffer_off..]);					// mov	(%rdx),%r8
	state.case_table_index = state.comp_buffer_pointer & 255;					// movzbq	(%rdx),%r9

	state.jmp_to = JUMP_CASE_TABLE;
    Jump
}

fn do_lzvn_10(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("jmpTable(10)\n");

	state.current_length = 
		state.length as u64 + state.case_table_index;							// lea	(%rax,%r9,1),%r11
	state.current_length += state.byte_count;									// add	%r10,%r11

	if state.current_length < state.decompressed_size as u64					// cmp	%rsi,%r11 (block_end: jae	Llzvn_l8)
	{
		write_u64!(state.decomp_buffer[state.length..], 
			state.comp_buffer_pointer);											// mov	%r8,(%rdi,%rax,1)
		state.length += state.case_table_index as usize;						// add	%r9,%rax
		state.comp_buffer_pointer = state.length as u64;						// mov	%rax,%r8
			
		if state.comp_buffer_pointer < state.negative_offset					// jb	Llzvn_exit
		{
			_LZVN_DEBUG_DUMP!("FAIL L10");
			return Exit(0);
		}

		state.comp_buffer_pointer -= state.negative_offset;						// sub	%r12,%r8

		if state.negative_offset < 8											// cmp	$0x8,%r12
		{
			state.jmp_to = LZVN_4;												// jb	Llzvn_l4
			return Jump;
		}

		state.jmp_to = LZVN_5;													// jmpq	*(%rbx,%r9,8)
		Jump
	} else {
		FallThrough
	}
}

const fn do_lzvn_11(state: &mut LZVNState) -> LZVNRes {
    _LZVN_DEBUG_DUMP!("jmpTable(11)\n");

	state.comp_buffer_pointer = state.length as u64;							// mov	%rax,%r8
	state.comp_buffer_pointer -= state.negative_offset;							// sub	%r12,%r8
	state.current_length = state.length as u64 + state.byte_count;				// lea	(%rax,%r10,1),%r11
	
	if (state.current_length < state.decompressed_size as u64) 
		&& (state.negative_offset >= 8) {
 		state.jmp_to = LZVN_5;													// jae	Llzvn_l5
 		return Jump;
 	}
	
	state.jmp_to = LZVN_4;														// jmp	Llzvn_l4
    Jump
}

#[allow(clippy::too_many_lines)]
pub fn lzvn_decode(decompressed_data: &mut [u8], compressed_data: &[u8]) -> usize {
	macro_rules! check_exit {
		($thing: expr) => {
			if let Exit(x) = $thing {
				return x
			}
		};
	}

    let mut state = LZVNState {
        decompressed_size: decompressed_data.len(),
        compressed_size: compressed_data.len(),
        decomp_buffer: decompressed_data,
        length: 0,              												// xor	%rax,%rax
		comp_buffer: compressed_data,
        comp_buffer_off: 0,
        comp_buffer_pointer: 0,
	    case_table_index: 0,
	    byte_count: 0,
	    current_length: 0,       												// xor	%r12,%r12
	    negative_offset: 0,
	    address: 0,             		// ((uint64_t)compBuffer + compBufferPointer)
	    jmp_to: JUMP_CASE_TABLE       	// On the first run
    };

	// Example values:
	//
	// byteCount: 10,	negativeOffset: 28957,	length: 42205762, currentLength: 42205772, compBufferPointer: 42176805
	// byteCount: 152,	negativeOffset: 28957,	length: 42205772, currentLength: 42205924, compBufferPointer: 42176815
	// byteCount: 10,	negativeOffset: 7933,	length: 42205924, currentLength: 42205934, compBufferPointer: 42197991
	// byteCount: 45,	negativeOffset: 7933,	length: 42205934, currentLength: 42205979, compBufferPointer: 42198001
	// byteCount: 9,	negativeOffset: 64,		length: 42205979, currentLength: 42205988, compBufferPointer: 42205915
	// byteCount: 10,	negativeOffset: 8180,	length: 42205988, currentLength: 42205998, compBufferPointer: 42197808
	// byteCount: 59,	negativeOffset: 8180,	length: 42205998, currentLength: 42206057, compBufferPointer: 42197818
	// byteCount: 10,	negativeOffset: 359,	length: 42206057, currentLength: 42206067, compBufferPointer: 42205698
	// byteCount: 1,	negativeOffset: 359,	length: 42206067, currentLength: 42206068, compBufferPointer: 42205708
	// byteCount: 10,	negativeOffset: 29021,	length: 42206068, currentLength: 42206078, compBufferPointer: 42177047
	//
	// length + byteCount = currentLength
	// currentLength - (negativeOffset + byteCount) = compBufferPointer
	// length - negativeOffset = compBufferPointer

	state.decompressed_size -= 8;												// sub	$0x8,%rsi

	if state.decompressed_size < 8												// jb	Llzvn_exit
	{
		return 0;
	}

	state.compressed_size = 
		state.comp_buffer_off + state.compressed_size - 8;						// lea	-0x8(%rdx,%rcx,1),%rcx

	state.comp_buffer_pointer = 
		read_u64!(state.comp_buffer[state.comp_buffer_off..]);					// mov	(%rdx),%r8
	state.case_table_index = state.comp_buffer_pointer & 255;					// movzbq	(%rdx),%r9

	loop																		// jmpq	*(%rbx,%r9,8)
	{
		match state.jmp_to														// our jump table
		{
			JUMP_CASE_TABLE => { /******************************************************/
				match CASE_TABLE[(state.case_table_index as u8) as usize]
				{
					0 => check_exit!(do_case_table_0(&mut state)),
                    1 => check_exit!(do_case_table_1(&mut state)),
					2 => check_exit!(
							do_case_table_2(&mut state)
                            .map_fall(|| do_case_table_3(&mut state))
						),
                    3 => check_exit!(do_case_table_3(&mut state)),
                    4 => check_exit!(do_case_table_4(&mut state)),
                    5 => check_exit!(do_case_table_5(&mut state)),
                    6 => check_exit!(do_case_table_6(&mut state)),
                    7 => check_exit!(do_case_table_7(&mut state)),
                    8 => check_exit!(do_case_table_8(&mut state)),
                    9 => check_exit!(do_case_table_9(&mut state)),
                    10 => check_exit!(do_case_table_10(&mut state)),
					_ => { 
						_LZVN_DEBUG_DUMP!("default() caseTableIndex[{}]\n", state.case_table_index as u8); 
                    }
				}
			},
            LZVN_0 => check_exit!(
					do_lzvn_0(&mut state)
                    .map_fall(|| do_lzvn_1(&mut state))
				),
			LZVN_1 => check_exit!(do_lzvn_1(&mut state)),
			LZVN_2 => check_exit!(
					do_lzvn_2(&mut state)
                    .map_fall(|| do_lzvn_3(&mut state))
				),
			LZVN_3 => check_exit!(do_lzvn_3(&mut state)),
			LZVN_4 => check_exit!(
					do_lzvn_4(&mut state)
                    .map_fall(|| do_lzvn_9(&mut state))
				),
			LZVN_9 => check_exit!(do_lzvn_9(&mut state)),
			LZVN_5 => check_exit!(do_lzvn_5(&mut state)),
			LZVN_10 => check_exit!(
					do_lzvn_10(&mut state)
                    .map_fall(|| do_lzvn_8(&mut state))
                    .map_fall(|| do_lzvn_6(&mut state))
                    .map_fall(|| do_lzvn_7(&mut state))
				),
			LZVN_8 => check_exit!(
					do_lzvn_8(&mut state)
                    .map_fall(|| do_lzvn_6(&mut state))
                    .map_fall(|| do_lzvn_7(&mut state))
				),
			LZVN_6 => check_exit!(
					do_lzvn_6(&mut state)
                    .map_fall(|| do_lzvn_7(&mut state))
				),
			LZVN_7 => check_exit!(do_lzvn_7(&mut state)),
			LZVN_11 => check_exit!(do_lzvn_11(&mut state)),
			_ => {}
		}																		// switch (jmpq)

	}
}

