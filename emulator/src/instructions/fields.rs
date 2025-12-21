use modular_bitfield::prelude::*;

macro_rules! precisely_sized_enum {
    // precisely_size_enum! { name = Foo, bits = 1, attrs = [ #[derive(Copy, Clone)] ], options = [ Bar, Baz = 1 ] }
    {
        $(#[$doc:meta])*
        name = $typename:ident,
        bits = $bits:expr,
        attrs = [$(#[$attr:meta]),*],
        options = [$($name:ident $(= $value:expr)?),* $(,)?] $(,)?
    } => {
        $(#[$doc])*
        #[derive(Specifier)]
        #[bits = $bits]
        $(#[$attr])*
        pub enum $typename {
            $($name $(= $value)?),*
        }
    };

    {
        $(#[$doc:meta])*
        name = $typename:ident,
        bits = $bits:expr,
        options = [$($name:ident $(= $value:expr)?),* $(,)?] $(,)?
    } => {
        precisely_sized_enum! {
            $(#[$doc])*
            name = $typename,
            bits = $bits,
            attrs = [
                #[derive(Debug, PartialEq, Eq, Copy, Clone)],
                #[must_use]
            ],
            options = [$($name $(= $value)?),*]
        }
    };
}

precisely_sized_enum! {
    #[doc = "Condition codes for conditional execution of instructions."]
    name = Condition,
    bits = 4,
    options = [
        // Equal
        EQ = 0b0000,

        // Not Equal
        NE = 0b0001,

        // Unsigned higher or same
        HS = 0b0010,

        // Unsigned lower
        LO = 0b0011,

        // Minus / NegativeMinus / Negative
        MI = 0b0100,

        // Plus / Positive or Zero
        PL = 0b0101,

        // Overflow
        VS = 0b0110,

        // No Overflow
        VC = 0b0111,

        // Unsigned Higher
        HI = 0b1000,

        // Unsigned Lower or Same
        LS = 0b1001,

        // Signed Greater Than or Equal
        GE = 0b1010,

        // Signed Less Than
        LT = 0b1011,

        // Signed Greater Than
        GT = 0b1100,

        // Signed Less Than or Equal
        LE = 0b1101,

        // Always
        AL = 0b1110,

        // Never
        NV = 0b1111
    ],
}

precisely_sized_enum! {
    #[doc = "Opcodes for data processing instructions."]
    #[doc = "[Offical documentation](https://developer.arm.com/documentation/ddi0403/d/Application-Level-Architecture/The-ARMv7-M-Instruction-Set/Data-processing-instructions/Standard-data-processing-instructions?lang=en)"]
    #[doc = "on what they each perform."]
    name = Opcode,
    bits = 4,
    options = [
        AND = 0b0000,
        EOR = 0b0001,
        SUB = 0b0010,
        RSB = 0b0011,
        ADD = 0b0100,
        ADC = 0b0101,
        SBC = 0b0110,
        RSC = 0b0111,
        TST = 0b1000,
        TEQ = 0b1001,
        CMP = 0b1010,
        CMN = 0b1011,
        ORR = 0b1100,
        MOV = 0b1101,
        BIC = 0b1110,
        MVN = 0b1111,
    ],
}

precisely_sized_enum! {
    #[doc = "Indicates whether the second operand is an immediate value or a register."]
    name = ImmediateFlag,
    bits = 1,
    options = [
        Imm = 0b1,
        Register = 0b0,
    ],
}

precisely_sized_enum! {
    #[doc = "Indicates whether the instruction should update the condition flags."]
    name = SetFlags,
    bits = 1,
    options = [
        Yes = 0b1,
        No = 0b0,
    ],
}

precisely_sized_enum! {
    #[doc = "General purpose register."]
    #[doc = "[List of registers](https://developer.arm.com/documentation/ddi0403/d/System-Level-Architecture/System-Level-Programmers--Model/Registers)"]
    name = Register,
    bits = 4,
    options = [
        R0 = 0,
        R1 = 1,
        R2 = 2,
        R3 = 3,
        R4 = 4,
        R5 = 5,
        R6 = 6,
        R7 = 7,
        R8 = 8,
        R9 = 9,
        R10 = 10,
        R11 = 11,
        R12 = 12,
        // SP
        R13 = 13,
        // LR
        R14 = 14,
        // PC
        R15 = 15,
    ],
}

precisely_sized_enum! {
    name = IndexFlag,
    bits = 1,
    options = [
        Pre = 0b1,
        Post = 0b0,
    ],
}

precisely_sized_enum! {
    name = UpDownFlag,
    bits = 1,
    options = [
        Add = 0b1,
        Sub = 0b0,
    ],
}

precisely_sized_enum! {
    name = ByteWordFlag,
    bits = 1,
    options = [
        Byte = 0b1,
        Word = 0b0,
    ],
}

precisely_sized_enum! {
    name = WriteBackFlag,
    bits = 1,
    options = [
        Write = 0b1,
        NoModify = 0b0,
    ],
}

precisely_sized_enum! {
    name = LoadStoreFlag,
    bits = 1,
    options = [
        Load = 0b1,
        Store = 0b0,
    ],
}

precisely_sized_enum! {
    name = OffsetType,
    bits = 1,
    options = [
        Immediate = 0b0,
        Register = 0b1,
    ],
}

precisely_sized_enum! {
    name = ShiftType,
    bits = 2,
    options = [
        LSL = 0b00,
        LSR = 0b01,
        ASR = 0b10,
        ROR = 0b11,
    ],
}

precisely_sized_enum! {
    name = LinkFlag,
    bits = 1,
    options = [
        Yes = 0b1,
        No = 0b0,
    ],
}

precisely_sized_enum! {
    name = PrivilegeActionFlag,
    bits = 1,
    options = [
        PsrUpdateOrUserBank = 0b1,
        Normal = 0b0,
    ],
}

pub mod register_mask {
    macro_rules! create_register_mask {
        [$($r:ident),+ $(,)?] => {
            $(pub const $r : RegisterMask = RegisterMask(raw::$r));+;
        };
    }

    mod raw {
        macro_rules! create_register_mask {
            [$($k:ident = $v:expr),+ $(,)?] => {
                $(pub const $k  : u16 = $v);+;
            };
        }
        create_register_mask![
            R0 = 1 << 0,
            R1 = 1 << 1,
            R2 = 1 << 2,
            R3 = 1 << 3,
            R4 = 1 << 4,
            R5 = 1 << 5,
            R6 = 1 << 6,
            R7 = 1 << 7,
            R8 = 1 << 8,
            R9 = 1 << 9,
            R10 = 1 << 10,
            R11 = 1 << 11,
            R12 = 1 << 12,
            R13 = 1 << 13,
            SP = R13,
            R14 = 1 << 14,
            LR = R14,
            R15 = 1 << 15,
            PC = R15,
        ];
    }

    create_register_mask![
        R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, SP,
        R14, LR, R15, PC
    ];

    /// Register mask for block data transfer instructions.
    /// Each bit represents a register from R0 to R15.
    /// If a bit is set,
    /// the corresponding register is included in the transfer.
    /// For example, a mask of `0b0000_0000_0000_0011` indicates that R0 and R1 are included.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    #[must_use]
    pub struct RegisterMask(u16);

    impl From<super::Register> for RegisterMask {
        fn from(value: super::Register) -> Self {
            use super::Register as R;

            match value {
                R::R0 => R0,
                R::R1 => R1,
                R::R2 => R2,
                R::R3 => R3,
                R::R4 => R4,
                R::R5 => R5,
                R::R6 => R6,
                R::R7 => R7,
                R::R8 => R8,
                R::R9 => R9,
                R::R10 => R10,
                R::R11 => R11,
                R::R12 => R12,
                R::R13 => R13,
                R::R14 => R14,
                R::R15 => R15,
            }
        }
    }

    impl From<u16> for RegisterMask {
        fn from(value: u16) -> Self {
            Self(value)
        }
    }

    impl From<RegisterMask> for u16 {
        fn from(value: RegisterMask) -> Self {
            value.0
        }
    }

    impl std::ops::BitOr for RegisterMask {
        type Output = RegisterMask;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl std::ops::BitOrAssign for RegisterMask {
        fn bitor_assign(&mut self, rhs: Self) {
            self.0 |= rhs.0;
        }
    }
}

#[allow(unused_imports)]
pub use register_mask::RegisterMask;
