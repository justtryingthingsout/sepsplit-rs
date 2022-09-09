#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case,
non_upper_case_globals, unused_assignments, unused_mut)]

use c2rust_asm_casts::AsmCastTrait;
extern "C" {
    fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong)
              -> *mut libc::c_void;
}
pub type int64_t = libc::c_long;
pub type uint8_t = libc::c_uchar;
pub type uint64_t = libc::c_ulong;
pub type uintptr_t = libc::c_ulong;
pub type size_t = libc::c_ulong;
pub type __uint64_t = libc::c_ulong;


pub unsafe fn lzvn_decode(mut decompressedData: *mut libc::c_void,
                                     mut decompressedSize: size_t,
                                     mut compressedData: *const libc::c_void,
                                     mut compressedSize: size_t) -> size_t {
    let decompBuffer: uintptr_t =
        decompressedData as uintptr_t; // xor	%rax,%rax
    let mut length: size_t = 0 as libc::c_int as size_t; // use p(ointer)?
    let mut compBuffer: uintptr_t =
        compressedData as uintptr_t; // xor	%r12,%r12
    let mut compBufferPointer: uint64_t =
        0 as libc::c_int as
            uint64_t; // ((uint64_t)compBuffer + compBufferPointer)
    let mut caseTableIndex: uint64_t =
        0 as libc::c_int as uint64_t; // On the first run!
    let mut byteCount: uint64_t = 0 as libc::c_int as uint64_t;
    let mut currentLength: uint64_t = 0 as libc::c_int as uint64_t;
    let mut negativeOffset: uint64_t = 0 as libc::c_int as uint64_t;
    let mut address: uintptr_t = 0 as libc::c_int as uintptr_t;
    let mut jmpTo: uint8_t = 127 as libc::c_int as uint8_t;
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
    static caseTable: [u8; 256] = [1, 1, 1, 1, 1, 1, 2, 3, 1, 1, 1, 1, 1, 1, 4, 3, 1, 1, 1, 1,
        1, 1, 4, 3, 1, 1, 1, 1, 1, 1, 5, 3, 1, 1, 1, 1, 1, 1, 5, 3, 1, 1, 1, 1, 1, 1, 5, 3,
        1, 1, 1, 1, 1, 1, 5, 3, 1, 1, 1, 1, 1, 1, 5, 3, 1, 1, 1, 1, 1, 1, 0, 3, 1, 1, 1, 1,
        1, 1, 0, 3, 1, 1, 1, 1, 1, 1, 0, 3, 1, 1, 1, 1, 1, 1, 0, 3, 1, 1, 1, 1, 1, 1, 0, 3,
        1, 1, 1, 1, 1, 1, 0, 3, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 1, 1, 1, 1,
        1, 1, 0, 3, 1, 1, 1, 1, 1, 1, 0, 3, 1, 1, 1, 1, 1, 1, 0, 3, 1, 1, 1, 1, 1, 1, 0, 3,
        6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        6, 6, 6, 6, 1, 1, 1, 1, 1, 1, 0, 3, 1, 1, 1, 1, 1, 1, 0, 3, 5, 5, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 9, 10, 10,
        10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10];

    decompressedSize =
        (decompressedSize as
            libc::c_ulong).wrapping_sub(8 as libc::c_int as libc::c_ulong) as
            size_t as size_t;
    if decompressedSize < 8 as libc::c_int as libc::c_ulong {
        // jb	Llzvn_exit
        return 0 as libc::c_int as size_t
    } // lea	-0x8(%rdx,%rcx,1),%rcx
    compressedSize =
        compBuffer.wrapping_add(compressedSize).wrapping_sub(8 as libc::c_int
            as
            libc::c_ulong);
    if compBuffer > compressedSize {
        // cmp	%rcx,%rdx
        return 0 as libc::c_int as size_t
        // ja	Llzvn_exit
    } // mov	(%rdx),%r8
    compBufferPointer = *(compBuffer as *mut uint64_t); // movzbq	(%rdx),%r9
    caseTableIndex = compBufferPointer & 255 as libc::c_int as libc::c_ulong;
    loop  {
        // jmpq	*(%rbx,%r9,8)
        let mut current_block_217: u64;
        match jmpTo as libc::c_int {
            127 => {
                // our jump table
                /* *****************************************************/
                match caseTable[caseTableIndex as uint8_t as usize] as
                    libc::c_int {
                    0 => {
                        caseTableIndex >>= 6 as libc::c_int; // shr	$0x6,%r9
                        compBuffer =
                            compBuffer.wrapping_add(caseTableIndex).wrapping_add(1
                                as
                                libc::c_int
                                as
                                libc::c_ulong); // lea	0x1(%rdx,%r9,1),%rdx
                        if compBuffer > compressedSize {
                            // cmp	%rcx,%rdx
                            return 0 as libc::c_int as size_t
                            // ja	Llzvn_exit
                        } // mov	$0x38,%r10
                        byteCount =
                            56 as libc::c_int as uint64_t; // and	%r8,%r10
                        byteCount &= compBufferPointer; // shr	$0x8,%r8
                        compBufferPointer >>=
                            8 as libc::c_int; // shr	$0x3,%r10
                        byteCount >>= 3 as libc::c_int; // add	$0x3,%r10
                        byteCount =
                            (byteCount as
                                libc::c_ulong).wrapping_add(3 as libc::c_int
                                as
                                libc::c_ulong)
                                as uint64_t as uint64_t; // jmp	Llzvn_l10
                        jmpTo = 10 as libc::c_int as uint8_t
                    }
                    1 => {
                        caseTableIndex >>= 6 as libc::c_int; // shr	$0x6,%r9
                        compBuffer =
                            compBuffer.wrapping_add(caseTableIndex).wrapping_add(2
                                as
                                libc::c_int
                                as
                                libc::c_ulong); // lea	0x2(%rdx,%r9,1),%rdx
                        if compBuffer > compressedSize {
                            // cmp	%rcx,%rdx
                            return 0 as libc::c_int as size_t
                            // ja	Llzvn_exit
                        } // mov	%r8,%r12
                        negativeOffset = compBufferPointer; // bswap	%r12
                        negativeOffset = {
                                let mut __v: __uint64_t = 0; // mov	%r12,%r10
                                let mut __x: __uint64_t =
                                    negativeOffset; // shl	$0x5,%r12
                                if 0 != 0 {
                                    __v =
                                        ((__x as libc::c_ulonglong &
                                            0xff00000000000000 as
                                                libc::c_ulonglong) >>
                                            56 as libc::c_int |
                                            (__x as libc::c_ulonglong &
                                                0xff000000000000 as
                                                    libc::c_ulonglong) >>
                                                40 as libc::c_int |
                                            (__x as libc::c_ulonglong &
                                                0xff0000000000 as
                                                    libc::c_ulonglong) >>
                                                24 as libc::c_int |
                                            (__x as libc::c_ulonglong &
                                                0xff00000000 as
                                                    libc::c_ulonglong) >>
                                                8 as libc::c_int |
                                            (__x as libc::c_ulonglong &
                                                0xff000000 as
                                                    libc::c_ulonglong) <<
                                                8 as libc::c_int |
                                            (__x as libc::c_ulonglong &
                                                0xff0000 as
                                                    libc::c_ulonglong) <<
                                                24 as libc::c_int |
                                            (__x as libc::c_ulonglong &
                                                0xff00 as
                                                    libc::c_ulonglong) <<
                                                40 as libc::c_int |
                                            (__x as libc::c_ulonglong &
                                                0xff as libc::c_ulonglong)
                                                << 56 as libc::c_int) as
                                            __uint64_t
                                } else {
                                    let fresh0 = &mut __v; // shl	$0x2,%r10
                                    let fresh1; // shr	$0x35,%r12
                                    let fresh2 = __x; // shr	$0x3d,%r10
                                    fresh1 = std::intrinsics::bswap(c2rust_asm_casts::AsmCast::cast_in(fresh0, fresh2));

                                    c2rust_asm_casts::AsmCast::cast_out(fresh0,
                                                                        fresh2,
                                                                        fresh1); // add	$0x3,%r10
                                } // jmp	Llzvn_l10
                                __v
                            }; // shr	$0x6,%r9
                        byteCount =
                            negativeOffset; // lea	0x3(%rdx,%r9,1),%rdx
                        negativeOffset <<= 5 as libc::c_int;
                        byteCount <<= 2 as libc::c_int;
                        negativeOffset >>= 53 as libc::c_int;
                        byteCount >>= 61 as libc::c_int;
                        compBufferPointer >>= 16 as libc::c_int;
                        byteCount =
                            (byteCount as
                                libc::c_ulong).wrapping_add(3 as libc::c_int
                                as
                                libc::c_ulong)
                                as uint64_t as uint64_t;
                        jmpTo = 10 as libc::c_int as uint8_t
                    }
                    2 => { return length }
                    3 => {
                        caseTableIndex >>= 6 as libc::c_int;
                        compBuffer =
                            compBuffer.wrapping_add(caseTableIndex).wrapping_add(3
                                as
                                libc::c_int
                                as
                                libc::c_ulong);
                        if compBuffer > compressedSize {
                            // cmp	%rcx,%rdx
                            return 0 as libc::c_int as size_t
                            // ja	Llzvn_exit
                        } // mov	$0x38,%r10
                        byteCount =
                            56 as libc::c_int as uint64_t; // mov	$0xffff,%r12
                        negativeOffset =
                            65535 as libc::c_int as uint64_t; // and	%r8,%r10
                        byteCount &= compBufferPointer; // shr	$0x8,%r8
                        compBufferPointer >>=
                            8 as libc::c_int; // shr	$0x3,%r10
                        byteCount >>= 3 as libc::c_int; // and	%r8,%r12
                        negativeOffset &= compBufferPointer; // shr	$0x10,%r8
                        compBufferPointer >>=
                            16 as libc::c_int; // add	$0x3,%r10
                        byteCount =
                            (byteCount as
                                libc::c_ulong).wrapping_add(3 as libc::c_int
                                as
                                libc::c_ulong)
                                as uint64_t as uint64_t; // jmp	Llzvn_l10
                        jmpTo = 10 as libc::c_int as uint8_t
                    }
                    4 => {
                        compBuffer =
                            compBuffer.wrapping_add(1); // add	$0x1,%rdx
                        if compBuffer > compressedSize {
                            // cmp	%rcx,%rdx
                            return 0 as libc::c_int as size_t
                            // ja	Llzvn_exit
                        } // mov	(%rdx),%r8
                        compBufferPointer =
                            *(compBuffer as
                                *mut uint64_t); // movzbq (%rdx),%r9
                        caseTableIndex =
                            compBufferPointer &
                                255 as libc::c_int as
                                    libc::c_ulong; // continue;
                        jmpTo = 127 as libc::c_int as uint8_t
                    }
                    5 => { return 0 as libc::c_int as size_t }
                    6 => { // Llzvn_table5;
                        caseTableIndex >>= 3 as libc::c_int; // shr	$0x3,%r9
                        caseTableIndex &=
                            3 as libc::c_int as libc::c_ulong; // and	$0x3,%r9
                        compBuffer =
                            compBuffer.wrapping_add(caseTableIndex).wrapping_add(3
                                as
                                libc::c_int
                                as
                                libc::c_ulong); // lea	0x3(%rdx,%r9,1),%rdx
                        if compBuffer > compressedSize {
                            // cmp	%rcx,%rdx
                            return 0 as libc::c_int as size_t
                            // ja	Llzvn_exit
                        } // mov	%r8,%r10
                        byteCount = compBufferPointer; // and	$0x307,%r10
                        byteCount &=
                            775 as libc::c_int as
                                libc::c_ulong; // shr	$0xa,%r8
                        compBufferPointer >>=
                            10 as libc::c_int; // movzbq %r10b,%r12
                        negativeOffset =
                            byteCount &
                                255 as libc::c_int as
                                    libc::c_ulong; // shr	$0x8,%r10
                        byteCount >>= 8 as libc::c_int; // shl	$0x2,%r12
                        negativeOffset <<= 2 as libc::c_int; // or	%r12,%r10
                        byteCount |= negativeOffset; // mov	$0x3fff,%r12
                        negativeOffset =
                            16383 as libc::c_int as uint64_t; // add	$0x3,%r10
                        byteCount =
                            (byteCount as
                                libc::c_ulong).wrapping_add(3 as libc::c_int
                                as
                                libc::c_ulong)
                                as uint64_t as uint64_t; // and	%r8,%r12
                        negativeOffset &= compBufferPointer; // shr	$0xe,%r8
                        compBufferPointer >>=
                            14 as libc::c_int; // jmp	Llzvn_l10
                        jmpTo = 10 as libc::c_int as uint8_t
                    }
                    7 => {
                        compBufferPointer >>=
                            8 as libc::c_int; // shr	$0x8,%r8
                        compBufferPointer &=
                            255 as libc::c_int as
                                libc::c_ulong; // and	$0xff,%r8
                        compBufferPointer =
                            (compBufferPointer as
                                libc::c_ulong).wrapping_add(16 as libc::c_int
                                as
                                libc::c_ulong)
                                as uint64_t as uint64_t; // add	$0x10,%r8
                        compBuffer =
                            compBuffer.wrapping_add(compBufferPointer).wrapping_add(2
                                as
                                libc::c_int
                                as
                                libc::c_ulong); // lea	0x2(%rdx,%r8,1),%rdx
                        jmpTo = 0 as libc::c_int as uint8_t
                    }
                    8 => { // jmp	Llzvn_l0
                        compBufferPointer &=
                            15 as libc::c_int as
                                libc::c_ulong; // and	$0xf,%r8
                        compBuffer =
                            compBuffer.wrapping_add(compBufferPointer).wrapping_add(1
                                as
                                libc::c_int
                                as
                                libc::c_ulong); // lea	0x1(%rdx,%r8,1),%rdx
                        jmpTo = 0 as libc::c_int as uint8_t
                    }
                    9 => { // jmp	Llzvn_l0
                        compBuffer =
                            (compBuffer as
                                libc::c_ulong).wrapping_add(2 as libc::c_int
                                as
                                libc::c_ulong)
                                as uintptr_t as uintptr_t; // add	$0x2,%rdx
                        if compBuffer > compressedSize {
                            // cmp	%rcx,%rdx
                            return 0 as libc::c_int as size_t
                            // ja	Llzvn_exit
                        }
                        // Up most significant byte (count) by 16 (0x10/16 - 0x10f/271).
                        byteCount = compBufferPointer; // mov	%r8,%r10
                        byteCount >>= 8 as libc::c_int; // shr	$0x8,%r10
                        byteCount &=
                            255 as libc::c_int as
                                libc::c_ulong; // and	$0xff,%r10
                        byteCount =
                            (byteCount as
                                libc::c_ulong).wrapping_add(16 as libc::c_int
                                as
                                libc::c_ulong)
                                as uint64_t as uint64_t; // add	$0x10,%r10
                        jmpTo = 11 as libc::c_int as uint8_t
                    }
                    10 => { // jmp	Llzvn_l11
                        compBuffer =
                            compBuffer.wrapping_add(1); // add	$0x1,%rdx
                        if compBuffer > compressedSize {
                            // cmp	%rcx,%rdx
                            return 0 as libc::c_int as size_t
                            // ja	Llzvn_exit
                        } // mov	%r8,%r10
                        byteCount = compBufferPointer; // and	$0xf,%r10
                        byteCount &=
                            15 as libc::c_int as
                                libc::c_ulong; // jmp	Llzvn_l11
                        jmpTo = 11 as libc::c_int as uint8_t
                    }
                    _ => { }
                }
                current_block_217 = 6530401058219605690;
            }
            0 => {
                /* *********************************************************/
                if compBuffer > compressedSize {
                    // cmp	%rcx,%rdx
                    return 0 as libc::c_int as size_t
                    // ja	Llzvn_exit
                } // lea	(%rax,%r8,1),%r11
                currentLength =
                    length.wrapping_add(compBufferPointer); // neg	%r8
                compBufferPointer = compBufferPointer.wrapping_neg();
                if currentLength > decompressedSize {
                    // cmp	%rsi,%r11
                    jmpTo = 2 as libc::c_int as uint8_t; // ja	Llzvn_l2
                    current_block_217 =
                        6530401058219605690; // lea	(%rdi,%r11,1),%r11
                } else {
                    currentLength = decompBuffer.wrapping_add(currentLength);
                    current_block_217 = 17289726979865446646;
                }
            }
            1 => { current_block_217 = 17289726979865446646; }
            2 => {
                /* *********************************************************/
                currentLength =
                    decompressedSize.wrapping_add(8 as libc::c_int as
                        libc::c_ulong); // lea	0x8(%rsi),%r11
                current_block_217 = 1852620334796398428;
            }
            3 => { current_block_217 = 1852620334796398428; }
            4 => {
                /* *********************************************************/
                currentLength =
                    decompressedSize.wrapping_add(8 as libc::c_int as
                        libc::c_ulong); // lea	0x8(%rsi),%r11
                current_block_217 = 2023619688134929128; // ja	Llzvn_l5
            }
            9 => { current_block_217 = 2023619688134929128; }
            5 => {
                loop
                /* *********************************************************/
                // Llzvn_l5: (block copy of qwords)
                {
                    address =
                        decompBuffer.wrapping_add(compBufferPointer); // mov	(%rdi,%r8,1),%r9
                    caseTableIndex =
                        *(address as *mut uint64_t); // add	$0x8,%r8
                    compBufferPointer =
                        (compBufferPointer as
                            libc::c_ulong).wrapping_add(8 as libc::c_int as
                            libc::c_ulong) as
                            uint64_t as uint64_t; // mov	%r9,(%rdi,%rax,1)
                    memcpy((decompBuffer as
                        *mut libc::c_char).offset(length as isize) as
                               *mut libc::c_void,
                           &mut caseTableIndex as *mut uint64_t as
                               *const libc::c_void,
                           8 as libc::c_int as
                               libc::c_ulong); // add	$0x8,%rax
                    length =
                        (length as
                            libc::c_ulong).wrapping_add(8 as libc::c_int as
                            libc::c_ulong) as
                            size_t as size_t; // add	%r10,%rax
                    byteCount =
                        (byteCount as
                            libc::c_ulong).wrapping_sub(8 as libc::c_int as
                            libc::c_ulong) as
                            uint64_t as uint64_t; // mov	(%rdx),%r8
                    if !(byteCount.wrapping_add(8 as libc::c_int as
                        libc::c_ulong) >
                        8 as libc::c_int as libc::c_ulong) {
                        break ; // movzbq	(%rdx),%r9
                    }
                } // jmpq	*(%rbx,%r9,8)
                length =
                    (length as libc::c_ulong).wrapping_add(byteCount) as
                        size_t as size_t;
                compBufferPointer = *(compBuffer as *mut uint64_t);
                caseTableIndex =
                    compBufferPointer & 255 as libc::c_int as libc::c_ulong;
                jmpTo = 127 as libc::c_int as uint8_t;
                current_block_217 = 6530401058219605690;
            }
            10 => {
                /* ********************************************************/
                currentLength =
                    length.wrapping_add(caseTableIndex); // lea	(%rax,%r9,1),%r11
                currentLength =
                    (currentLength as libc::c_ulong).wrapping_add(byteCount)
                        as uint64_t as uint64_t; // add	%r10,%r11
                if currentLength < decompressedSize {
                    // cmp	%rsi,%r11 (block_end: jae	Llzvn_l8)
                    memcpy((decompBuffer as
                        *mut libc::c_char).offset(length as isize) as
                               *mut libc::c_void,
                           &mut compBufferPointer as *mut uint64_t as
                               *const libc::c_void,
                           8 as libc::c_int as
                               libc::c_ulong); // mov	%r8,(%rdi,%rax,1)
                    length =
                        (length as libc::c_ulong).wrapping_add(caseTableIndex)
                            as size_t as size_t; // add	%r9,%rax
                    compBufferPointer = length; // mov	%rax,%r8
                    if compBufferPointer < negativeOffset {
                        // jb	Llzvn_exit
                        return 0 as libc::c_int as size_t
                    } // sub	%r12,%r8
                    compBufferPointer =
                        (compBufferPointer as
                            libc::c_ulong).wrapping_sub(negativeOffset) as
                            uint64_t as uint64_t;
                    if negativeOffset < 8 as libc::c_int as libc::c_ulong {
                        // cmp	$0x8,%r12
                        jmpTo = 4 as libc::c_int as uint8_t
                    } else { // jb	Llzvn_l4
                        jmpTo = 5 as libc::c_int as uint8_t
                    } // jmpq	*(%rbx,%r9,8)
                    current_block_217 = 6530401058219605690;
                } else { current_block_217 = 11322929247169729670; }
            }
            8 => { current_block_217 = 11322929247169729670; }
            6 => { current_block_217 = 18349474674938321835; }
            7 => { current_block_217 = 12153365054289215322; }
            11 => {
                /* ********************************************************/
                compBufferPointer = length; // mov	%rax,%r8
                compBufferPointer =
                    (compBufferPointer as
                        libc::c_ulong).wrapping_sub(negativeOffset) as
                        uint64_t as uint64_t; // sub	%r12,%r8
                currentLength =
                    length.wrapping_add(byteCount); // lea	(%rax,%r10,1),%r11
                if currentLength < decompressedSize {
                    // cmp	%rsi,%r11
                    if negativeOffset >= 8 as libc::c_int as libc::c_ulong {
                        // cmp	$0x8,%r12
                        jmpTo = 5 as libc::c_int as uint8_t; // jae	Llzvn_l5
                        current_block_217 =
                            6530401058219605690; // jmp	Llzvn_l4
                    } else { current_block_217 = 9728093949049737828; }
                } else { current_block_217 = 9728093949049737828; }
                match current_block_217 {
                    6530401058219605690 => { }
                    _ => {
                        jmpTo = 4 as libc::c_int as uint8_t;
                        current_block_217 = 6530401058219605690;
                    }
                }
            }
            _ => { current_block_217 = 6530401058219605690; }
        }
        match current_block_217 {
            11322929247169729670 =>
            /* *********************************************************/
                {
                    if caseTableIndex == 0 as libc::c_int as libc::c_ulong {
                        // test	%r9,%r9
                        jmpTo = 7 as libc::c_int as uint8_t; // jmpq	*(%rbx,%r9,8)
                        current_block_217 =
                            6530401058219605690; // lea	0x8(%rsi),%r11
                    } else {
                        currentLength =
                            decompressedSize.wrapping_add(8 as libc::c_int as
                                libc::c_ulong); // jne	Llzvn_l9
                        current_block_217 = 18349474674938321835;
                    }
                }
            2023619688134929128 => {
                loop
                /* *********************************************************/
                // Llzvn_l9: (block copy of bytes)
                {
                    address =
                        decompBuffer.wrapping_add(compBufferPointer); // movzbq (%rdi,%r8,1),%r9
                    caseTableIndex =
                        (*(address as *mut uint8_t) as libc::c_int &
                            255 as libc::c_int) as uint64_t; // add	$0x1,%r8
                    compBufferPointer =
                        compBufferPointer.wrapping_add(1); // mov	%r9,(%rdi,%rax,1)
                    memcpy((decompBuffer as
                        *mut libc::c_char).offset(length as isize) as
                               *mut libc::c_void,
                           &mut caseTableIndex as *mut uint64_t as
                               *const libc::c_void,
                           1 as libc::c_int as
                               libc::c_ulong); // add	$0x1,%rax
                    length = length.wrapping_add(1);
                    if length == currentLength {
                        // cmp	%rax,%r11
                        return length
                        // je	Llzvn_exit2
                    } // mov	(%rdx),%r8
                    byteCount =
                        byteCount.wrapping_sub(1); // movzbq	(%rdx),%r9
                    if !(byteCount != 0) {
                        break ; // jmpq	*(%rbx,%r9,8)
                    }
                } // jne	Llzvn_l3
                compBufferPointer = *(compBuffer as *mut uint64_t);
                caseTableIndex =
                    compBufferPointer & 255 as libc::c_int as libc::c_ulong;
                jmpTo = 127 as libc::c_int as uint8_t;
                current_block_217 = 6530401058219605690;
            }
            1852620334796398428 => {
                loop
                /* **********************************************************/
                // Llzvn_l3: (block copy of bytes)
                {
                    address =
                        compBuffer.wrapping_add(compBufferPointer); // movzbq (%rdx,%r8,1),%r9
                    caseTableIndex =
                        *(address as *mut uint64_t) &
                            255 as libc::c_int as
                                libc::c_ulong; // add	$0x1,%rax
                    memcpy((decompBuffer as
                        *mut libc::c_char).offset(length as isize) as
                               *mut libc::c_void,
                           &mut caseTableIndex as *mut uint64_t as
                               *const libc::c_void,
                           1 as libc::c_int as libc::c_ulong);
                    length = length.wrapping_add(1);
                    if currentLength == length {
                        // cmp	%rax,%r11
                        return length
                        // je	Llzvn_exit2
                    } // mov	(%rdx),%r8
                    compBufferPointer =
                        compBufferPointer.wrapping_add(1); // movzbq	(%rdx),%r9
                    if !(compBufferPointer as int64_t !=
                        0 as libc::c_int as libc::c_long) {
                        break ; // jmpq	*(%rbx,%r9,8)
                    }
                } // jae	Llzvn_l1
                compBufferPointer = *(compBuffer as *mut uint64_t);
                caseTableIndex =
                    compBufferPointer & 255 as libc::c_int as libc::c_ulong;
                jmpTo = 127 as libc::c_int as uint8_t;
                current_block_217 = 6530401058219605690;
            }
            17289726979865446646 => {
                loop
                /* *********************************************************/
                // Llzvn_l1:
                //					caseTableIndex = *(uint64_t *)((uint64_t)compBuffer + compBufferPointer);
                {
                    address =
                        compBuffer.wrapping_add(compBufferPointer); // mov	(%rdx,%r8,1),%r9
                    caseTableIndex = *(address as *mut uint64_t);
                    address = currentLength.wrapping_add(compBufferPointer);
                    *(address as *mut uint64_t) = caseTableIndex;
                    compBufferPointer =
                        (compBufferPointer as
                            libc::c_ulong).wrapping_add(8 as libc::c_int as
                            libc::c_ulong) as
                            uint64_t as uint64_t;
                    if !((18446744073709551615 as
                        libc::c_ulong).wrapping_sub(compBufferPointer.wrapping_sub(8
                        as
                        libc::c_int
                        as
                        libc::c_ulong))
                        >= 8 as libc::c_int as libc::c_ulong) {
                        break ;
                    }
                }
                //					*(uint64_t *)((uint64_t)currentLength + compBufferPointer) = caseTableIndex;
// or:
//					memcpy((void *)currentLength + compBufferPointer, &caseTableIndex, 8);
// or:
                // mov	%r9,(%r11,%r8,1)
                length = currentLength; // mov	%r11,%rax
                length =
                    (length as libc::c_ulong).wrapping_sub(decompBuffer) as
                        size_t as size_t; // sub	%rdi,%rax
                compBufferPointer =
                    *(compBuffer as *mut uint64_t); // mov	(%rdx),%r8
                caseTableIndex =
                    compBufferPointer &
                        255 as libc::c_int as
                            libc::c_ulong; // movzbq (%rdx),%r9
                jmpTo = 127 as libc::c_int as uint8_t; // jmpq	*(%rbx,%r9,8)
                current_block_217 = 6530401058219605690; // jne	Llzvn_l6
            }
            _ => { }
        }
        match current_block_217 {
            18349474674938321835 => {
                loop
                /* *********************************************************/
                {
                    memcpy((decompBuffer as
                        *mut libc::c_char).offset(length as isize) as
                               *mut libc::c_void,
                           &mut compBufferPointer as *mut uint64_t as
                               *const libc::c_void,
                           1 as libc::c_int as
                               libc::c_ulong); // mov	%r8b,(%rdi,%rax,1)
                    // sub	$0x1,%r9
                    length = length.wrapping_add(1); // add	$0x1,%rax
                    if length == currentLength {
                        // cmp	%rax,%r11
                        return length
                        // je	Llzvn_exit2
                    } // shr	$0x8,%r8
                    compBufferPointer >>= 8 as libc::c_int;
                    caseTableIndex = caseTableIndex.wrapping_sub(1);
                    if !(caseTableIndex != 1 as libc::c_int as libc::c_ulong)
                    {
                        break ;
                    }
                }
                current_block_217 = 12153365054289215322;
            }
            _ => { }
        }
        match current_block_217 {
            12153365054289215322 =>
            /* *********************************************************/
                {
                    compBufferPointer = length; // mov	%rax,%r8
                    compBufferPointer =
                        (compBufferPointer as
                            libc::c_ulong).wrapping_sub(negativeOffset) as
                            uint64_t as uint64_t; // sub	%r12,%r8
                    if compBufferPointer < negativeOffset {
                        // jb	Llzvn_exit
                        return 0 as libc::c_int as size_t
                    }
                    jmpTo = 4 as libc::c_int as uint8_t
                }
            _ => { }
        }
        // switch (jmpq)
    };
}
