#[derive(Clone, Hash)]
/// Flags group `riscv64`.
pub struct Flags {
    bytes: [u8; 4],
}
impl Flags {
    /// Create flags riscv64 settings group.
    #[allow(unused_variables)]
    pub fn new(shared: &settings::Flags, builder: &Builder) -> Self {
        let bvec = builder.state_for("riscv64");
        let mut riscv64 = Self { bytes: [0; 4] };
        debug_assert_eq!(bvec.len(), 4);
        riscv64.bytes[0..4].copy_from_slice(&bvec);
        // Precompute #29.
        if riscv64.has_m() && riscv64.has_a() && riscv64.has_f() && riscv64.has_d() && riscv64.has_zicsr() && riscv64.has_zifencei() {
            riscv64.bytes[3] |= 1 << 5;
        }
        riscv64
    }
}
impl Flags {
    /// Iterates the setting values.
    pub fn iter(&self) -> impl Iterator<Item = Value> {
        let mut bytes = [0; 4];
        bytes.copy_from_slice(&self.bytes[0..4]);
        DESCRIPTORS.iter().filter_map(move |d| {
            let values = match &d.detail {
                detail::Detail::Preset => return None,
                detail::Detail::Enum { last, enumerators } => Some(TEMPLATE.enums(*last, *enumerators)),
                _ => None
            };
            Some(Value{ name: d.name, detail: d.detail, values, value: bytes[d.offset as usize] })
        })
    }
}
/// User-defined settings.
#[allow(dead_code)]
impl Flags {
    /// Get a view of the boolean predicates.
    pub fn predicate_view(&self) -> crate::settings::PredicateView {
        crate::settings::PredicateView::new(&self.bytes[0..])
    }
    /// Dynamic numbered predicate getter.
    fn numbered_predicate(&self, p: usize) -> bool {
        self.bytes[0 + p / 8] & (1 << (p % 8)) != 0
    }
    /// has extension M?
    /// Integer multiplication and division
    pub fn has_m(&self) -> bool {
        self.numbered_predicate(0)
    }
    /// has extension A?
    /// Atomic instructions
    pub fn has_a(&self) -> bool {
        self.numbered_predicate(1)
    }
    /// has extension F?
    /// Single-precision floating point
    pub fn has_f(&self) -> bool {
        self.numbered_predicate(2)
    }
    /// has extension D?
    /// Double-precision floating point
    pub fn has_d(&self) -> bool {
        self.numbered_predicate(3)
    }
    /// has extension Zfa?
    /// Zfa: Extension for Additional Floating-Point Instructions
    pub fn has_zfa(&self) -> bool {
        self.numbered_predicate(4)
    }
    /// has extension V?
    /// Vector instruction support
    pub fn has_v(&self) -> bool {
        self.numbered_predicate(5)
    }
    /// has extension Zca?
    /// Zca is the C extension without floating point loads
    pub fn has_zca(&self) -> bool {
        self.numbered_predicate(6)
    }
    /// has extension Zcd?
    /// Zcd contains only the double precision floating point loads from the C extension
    pub fn has_zcd(&self) -> bool {
        self.numbered_predicate(7)
    }
    /// has extension Zcb?
    /// Zcb: Extra compressed instructions
    pub fn has_zcb(&self) -> bool {
        self.numbered_predicate(8)
    }
    /// has extension zbkb?
    /// Zbkb: Bit-manipulation for Cryptography
    pub fn has_zbkb(&self) -> bool {
        self.numbered_predicate(9)
    }
    /// has extension zba?
    /// Zba: Address Generation
    pub fn has_zba(&self) -> bool {
        self.numbered_predicate(10)
    }
    /// has extension zbb?
    /// Zbb: Basic bit-manipulation
    pub fn has_zbb(&self) -> bool {
        self.numbered_predicate(11)
    }
    /// has extension zbc?
    /// Zbc: Carry-less multiplication
    pub fn has_zbc(&self) -> bool {
        self.numbered_predicate(12)
    }
    /// has extension zbs?
    /// Zbs: Single-bit instructions
    pub fn has_zbs(&self) -> bool {
        self.numbered_predicate(13)
    }
    /// has extension zicond?
    /// ZiCond: Integer Conditional Operations
    pub fn has_zicond(&self) -> bool {
        self.numbered_predicate(14)
    }
    /// has extension zicsr?
    /// Zicsr: Control and Status Register (CSR) Instructions
    pub fn has_zicsr(&self) -> bool {
        self.numbered_predicate(15)
    }
    /// has extension zifencei?
    /// Zifencei: Instruction-Fetch Fence
    pub fn has_zifencei(&self) -> bool {
        self.numbered_predicate(16)
    }
    /// has extension Zvl32b?
    /// Zvl32b: Vector register has a minimum of 32 bits
    pub fn has_zvl32b(&self) -> bool {
        self.numbered_predicate(17)
    }
    /// has extension Zvl64b?
    /// Zvl64b: Vector register has a minimum of 64 bits
    pub fn has_zvl64b(&self) -> bool {
        self.numbered_predicate(18)
    }
    /// has extension Zvl128b?
    /// Zvl128b: Vector register has a minimum of 128 bits
    pub fn has_zvl128b(&self) -> bool {
        self.numbered_predicate(19)
    }
    /// has extension Zvl256b?
    /// Zvl256b: Vector register has a minimum of 256 bits
    pub fn has_zvl256b(&self) -> bool {
        self.numbered_predicate(20)
    }
    /// has extension Zvl512b?
    /// Zvl512b: Vector register has a minimum of 512 bits
    pub fn has_zvl512b(&self) -> bool {
        self.numbered_predicate(21)
    }
    /// has extension Zvl1024b?
    /// Zvl1024b: Vector register has a minimum of 1024 bits
    pub fn has_zvl1024b(&self) -> bool {
        self.numbered_predicate(22)
    }
    /// has extension Zvl2048b?
    /// Zvl2048b: Vector register has a minimum of 2048 bits
    pub fn has_zvl2048b(&self) -> bool {
        self.numbered_predicate(23)
    }
    /// has extension Zvl4096b?
    /// Zvl4096b: Vector register has a minimum of 4096 bits
    pub fn has_zvl4096b(&self) -> bool {
        self.numbered_predicate(24)
    }
    /// has extension Zvl8192b?
    /// Zvl8192b: Vector register has a minimum of 8192 bits
    pub fn has_zvl8192b(&self) -> bool {
        self.numbered_predicate(25)
    }
    /// has extension Zvl16384b?
    /// Zvl16384b: Vector register has a minimum of 16384 bits
    pub fn has_zvl16384b(&self) -> bool {
        self.numbered_predicate(26)
    }
    /// has extension Zvl32768b?
    /// Zvl32768b: Vector register has a minimum of 32768 bits
    pub fn has_zvl32768b(&self) -> bool {
        self.numbered_predicate(27)
    }
    /// has extension Zvl65536b?
    /// Zvl65536b: Vector register has a minimum of 65536 bits
    pub fn has_zvl65536b(&self) -> bool {
        self.numbered_predicate(28)
    }
    /// Computed predicate `riscv64.has_m() && riscv64.has_a() && riscv64.has_f() && riscv64.has_d() && riscv64.has_zicsr() && riscv64.has_zifencei()`.
    pub fn has_g(&self) -> bool {
        self.numbered_predicate(29)
    }
}
static DESCRIPTORS: [detail::Descriptor; 42] = [
    detail::Descriptor {
        name: "has_m",
        description: "has extension M?",
        offset: 0,
        detail: detail::Detail::Bool { bit: 0 },
    },
    detail::Descriptor {
        name: "has_a",
        description: "has extension A?",
        offset: 0,
        detail: detail::Detail::Bool { bit: 1 },
    },
    detail::Descriptor {
        name: "has_f",
        description: "has extension F?",
        offset: 0,
        detail: detail::Detail::Bool { bit: 2 },
    },
    detail::Descriptor {
        name: "has_d",
        description: "has extension D?",
        offset: 0,
        detail: detail::Detail::Bool { bit: 3 },
    },
    detail::Descriptor {
        name: "has_zfa",
        description: "has extension Zfa?",
        offset: 0,
        detail: detail::Detail::Bool { bit: 4 },
    },
    detail::Descriptor {
        name: "has_v",
        description: "has extension V?",
        offset: 0,
        detail: detail::Detail::Bool { bit: 5 },
    },
    detail::Descriptor {
        name: "has_zca",
        description: "has extension Zca?",
        offset: 0,
        detail: detail::Detail::Bool { bit: 6 },
    },
    detail::Descriptor {
        name: "has_zcd",
        description: "has extension Zcd?",
        offset: 0,
        detail: detail::Detail::Bool { bit: 7 },
    },
    detail::Descriptor {
        name: "has_zcb",
        description: "has extension Zcb?",
        offset: 1,
        detail: detail::Detail::Bool { bit: 0 },
    },
    detail::Descriptor {
        name: "has_zbkb",
        description: "has extension zbkb?",
        offset: 1,
        detail: detail::Detail::Bool { bit: 1 },
    },
    detail::Descriptor {
        name: "has_zba",
        description: "has extension zba?",
        offset: 1,
        detail: detail::Detail::Bool { bit: 2 },
    },
    detail::Descriptor {
        name: "has_zbb",
        description: "has extension zbb?",
        offset: 1,
        detail: detail::Detail::Bool { bit: 3 },
    },
    detail::Descriptor {
        name: "has_zbc",
        description: "has extension zbc?",
        offset: 1,
        detail: detail::Detail::Bool { bit: 4 },
    },
    detail::Descriptor {
        name: "has_zbs",
        description: "has extension zbs?",
        offset: 1,
        detail: detail::Detail::Bool { bit: 5 },
    },
    detail::Descriptor {
        name: "has_zicond",
        description: "has extension zicond?",
        offset: 1,
        detail: detail::Detail::Bool { bit: 6 },
    },
    detail::Descriptor {
        name: "has_zicsr",
        description: "has extension zicsr?",
        offset: 1,
        detail: detail::Detail::Bool { bit: 7 },
    },
    detail::Descriptor {
        name: "has_zifencei",
        description: "has extension zifencei?",
        offset: 2,
        detail: detail::Detail::Bool { bit: 0 },
    },
    detail::Descriptor {
        name: "has_zvl32b",
        description: "has extension Zvl32b?",
        offset: 2,
        detail: detail::Detail::Bool { bit: 1 },
    },
    detail::Descriptor {
        name: "has_zvl64b",
        description: "has extension Zvl64b?",
        offset: 2,
        detail: detail::Detail::Bool { bit: 2 },
    },
    detail::Descriptor {
        name: "has_zvl128b",
        description: "has extension Zvl128b?",
        offset: 2,
        detail: detail::Detail::Bool { bit: 3 },
    },
    detail::Descriptor {
        name: "has_zvl256b",
        description: "has extension Zvl256b?",
        offset: 2,
        detail: detail::Detail::Bool { bit: 4 },
    },
    detail::Descriptor {
        name: "has_zvl512b",
        description: "has extension Zvl512b?",
        offset: 2,
        detail: detail::Detail::Bool { bit: 5 },
    },
    detail::Descriptor {
        name: "has_zvl1024b",
        description: "has extension Zvl1024b?",
        offset: 2,
        detail: detail::Detail::Bool { bit: 6 },
    },
    detail::Descriptor {
        name: "has_zvl2048b",
        description: "has extension Zvl2048b?",
        offset: 2,
        detail: detail::Detail::Bool { bit: 7 },
    },
    detail::Descriptor {
        name: "has_zvl4096b",
        description: "has extension Zvl4096b?",
        offset: 3,
        detail: detail::Detail::Bool { bit: 0 },
    },
    detail::Descriptor {
        name: "has_zvl8192b",
        description: "has extension Zvl8192b?",
        offset: 3,
        detail: detail::Detail::Bool { bit: 1 },
    },
    detail::Descriptor {
        name: "has_zvl16384b",
        description: "has extension Zvl16384b?",
        offset: 3,
        detail: detail::Detail::Bool { bit: 2 },
    },
    detail::Descriptor {
        name: "has_zvl32768b",
        description: "has extension Zvl32768b?",
        offset: 3,
        detail: detail::Detail::Bool { bit: 3 },
    },
    detail::Descriptor {
        name: "has_zvl65536b",
        description: "has extension Zvl65536b?",
        offset: 3,
        detail: detail::Detail::Bool { bit: 4 },
    },
    detail::Descriptor {
        name: "has_c",
        description: "Support for compressed instructions",
        offset: 0,
        detail: detail::Detail::Preset,
    },
    detail::Descriptor {
        name: "zvl32b",
        description: "Has a vector register size of at least 32 bits",
        offset: 4,
        detail: detail::Detail::Preset,
    },
    detail::Descriptor {
        name: "zvl64b",
        description: "Has a vector register size of at least 64 bits",
        offset: 8,
        detail: detail::Detail::Preset,
    },
    detail::Descriptor {
        name: "zvl128b",
        description: "Has a vector register size of at least 128 bits",
        offset: 12,
        detail: detail::Detail::Preset,
    },
    detail::Descriptor {
        name: "zvl256b",
        description: "Has a vector register size of at least 256 bits",
        offset: 16,
        detail: detail::Detail::Preset,
    },
    detail::Descriptor {
        name: "zvl512b",
        description: "Has a vector register size of at least 512 bits",
        offset: 20,
        detail: detail::Detail::Preset,
    },
    detail::Descriptor {
        name: "zvl1024b",
        description: "Has a vector register size of at least 1024 bits",
        offset: 24,
        detail: detail::Detail::Preset,
    },
    detail::Descriptor {
        name: "zvl2048b",
        description: "Has a vector register size of at least 2048 bits",
        offset: 28,
        detail: detail::Detail::Preset,
    },
    detail::Descriptor {
        name: "zvl4096b",
        description: "Has a vector register size of at least 4096 bits",
        offset: 32,
        detail: detail::Detail::Preset,
    },
    detail::Descriptor {
        name: "zvl8192b",
        description: "Has a vector register size of at least 8192 bits",
        offset: 36,
        detail: detail::Detail::Preset,
    },
    detail::Descriptor {
        name: "zvl16384b",
        description: "Has a vector register size of at least 16384 bits",
        offset: 40,
        detail: detail::Detail::Preset,
    },
    detail::Descriptor {
        name: "zvl32768b",
        description: "Has a vector register size of at least 32768 bits",
        offset: 44,
        detail: detail::Detail::Preset,
    },
    detail::Descriptor {
        name: "zvl65536b",
        description: "Has a vector register size of at least 65536 bits",
        offset: 48,
        detail: detail::Detail::Preset,
    },
];
static ENUMERATORS: [&str; 0] = [
];
static HASH_TABLE: [u16; 64] = [
    23,
    17,
    21,
    0xffff,
    39,
    41,
    27,
    4,
    11,
    8,
    6,
    10,
    12,
    0xffff,
    0xffff,
    7,
    0xffff,
    32,
    33,
    25,
    15,
    34,
    0xffff,
    0xffff,
    5,
    16,
    24,
    0xffff,
    0xffff,
    9,
    19,
    0xffff,
    30,
    0,
    20,
    29,
    28,
    1,
    31,
    0xffff,
    2,
    40,
    3,
    0xffff,
    0xffff,
    0xffff,
    0xffff,
    0xffff,
    0xffff,
    0xffff,
    0xffff,
    0xffff,
    35,
    22,
    26,
    14,
    0xffff,
    13,
    0xffff,
    0xffff,
    37,
    36,
    18,
    38,
];
static PRESETS: [(u8, u8); 52] = [
    // has_c: has_zca, has_zcd
    (0b11000000, 0b11000000),
    (0b00000000, 0b00000000),
    (0b00000000, 0b00000000),
    (0b00000000, 0b00000000),
    // zvl32b: has_zvl32b
    (0b00000000, 0b00000000),
    (0b00000000, 0b00000000),
    (0b00000010, 0b00000010),
    (0b00000000, 0b00000000),
    // zvl64b: has_zvl64b, has_zvl32b
    (0b00000000, 0b00000000),
    (0b00000000, 0b00000000),
    (0b00000110, 0b00000110),
    (0b00000000, 0b00000000),
    // zvl128b: has_zvl128b, has_zvl64b, has_zvl32b
    (0b00000000, 0b00000000),
    (0b00000000, 0b00000000),
    (0b00001110, 0b00001110),
    (0b00000000, 0b00000000),
    // zvl256b: has_zvl256b, has_zvl128b, has_zvl64b, has_zvl32b
    (0b00000000, 0b00000000),
    (0b00000000, 0b00000000),
    (0b00011110, 0b00011110),
    (0b00000000, 0b00000000),
    // zvl512b: has_zvl512b, has_zvl256b, has_zvl128b, has_zvl64b, has_zvl32b
    (0b00000000, 0b00000000),
    (0b00000000, 0b00000000),
    (0b00111110, 0b00111110),
    (0b00000000, 0b00000000),
    // zvl1024b: has_zvl1024b, has_zvl512b, has_zvl256b, has_zvl128b, has_zvl64b, has_zvl32b
    (0b00000000, 0b00000000),
    (0b00000000, 0b00000000),
    (0b01111110, 0b01111110),
    (0b00000000, 0b00000000),
    // zvl2048b: has_zvl2048b, has_zvl1024b, has_zvl512b, has_zvl256b, has_zvl128b, has_zvl64b, has_zvl32b
    (0b00000000, 0b00000000),
    (0b00000000, 0b00000000),
    (0b11111110, 0b11111110),
    (0b00000000, 0b00000000),
    // zvl4096b: has_zvl4096b, has_zvl2048b, has_zvl1024b, has_zvl512b, has_zvl256b, has_zvl128b, has_zvl64b, has_zvl32b
    (0b00000000, 0b00000000),
    (0b00000000, 0b00000000),
    (0b11111110, 0b11111110),
    (0b00000001, 0b00000001),
    // zvl8192b: has_zvl8192b, has_zvl4096b, has_zvl2048b, has_zvl1024b, has_zvl512b, has_zvl256b, has_zvl128b, has_zvl64b, has_zvl32b
    (0b00000000, 0b00000000),
    (0b00000000, 0b00000000),
    (0b11111110, 0b11111110),
    (0b00000011, 0b00000011),
    // zvl16384b: has_zvl16384b, has_zvl8192b, has_zvl4096b, has_zvl2048b, has_zvl1024b, has_zvl512b, has_zvl256b, has_zvl128b, has_zvl64b, has_zvl32b
    (0b00000000, 0b00000000),
    (0b00000000, 0b00000000),
    (0b11111110, 0b11111110),
    (0b00000111, 0b00000111),
    // zvl32768b: has_zvl32768b, has_zvl16384b, has_zvl8192b, has_zvl4096b, has_zvl2048b, has_zvl1024b, has_zvl512b, has_zvl256b, has_zvl128b, has_zvl64b, has_zvl32b
    (0b00000000, 0b00000000),
    (0b00000000, 0b00000000),
    (0b11111110, 0b11111110),
    (0b00001111, 0b00001111),
    // zvl65536b: has_zvl65536b, has_zvl32768b, has_zvl16384b, has_zvl8192b, has_zvl4096b, has_zvl2048b, has_zvl1024b, has_zvl512b, has_zvl256b, has_zvl128b, has_zvl64b, has_zvl32b
    (0b00000000, 0b00000000),
    (0b00000000, 0b00000000),
    (0b11111110, 0b11111110),
    (0b00011111, 0b00011111),
];
static TEMPLATE: detail::Template = detail::Template {
    name: "riscv64",
    descriptors: &DESCRIPTORS,
    enumerators: &ENUMERATORS,
    hash_table: &HASH_TABLE,
    defaults: &[0x0f, 0x80, 0x01, 0x00],
    presets: &PRESETS,
};
/// Create a `settings::Builder` for the riscv64 settings group.
pub fn builder() -> Builder {
    Builder::new(&TEMPLATE)
}
impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[riscv64]")?;
        for d in &DESCRIPTORS {
            if !d.detail.is_preset() {
                write!(f, "{} = ", d.name)?;
                TEMPLATE.format_toml_value(d.detail, self.bytes[d.offset as usize], f)?;
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
