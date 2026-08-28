// parser/cmd.rs
#[repr(u8)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Cmd {
    Add, Sub, Mul, Div, Mod,  // 0-4
    Shl, Shr, BitOr, BitAnd, Xor,  // 5-9
    Retc, Retv, PopRet,  // 10-12
    Jmp, IfJmp, ElseJump,  // 13-15
    // Cmp, JG, JE, JL, JNE, JGE, JLE
    Movc, Mov,  // 16-17
    Newv, Newp, Delv, Delp, Nop,  // 18-22
    Call,  // 243
    PushVar, PopVar, StoreVar, Dup, Ldc, Pop,  // 24-29
    ConvTop,  // 30
}

pub(crate) mod cmd_u8 {
    use super::Cmd;
    // arithmetic
    pub const ADD: u8 = Cmd::Add as u8;
    pub const SUB: u8 = Cmd::Sub as u8;
    pub const MUL: u8 = Cmd::Mul as u8;
    pub const DIV: u8 = Cmd::Div as u8;
    pub const MOD: u8 = Cmd::Mod as u8;
    // Bit
    pub const SHL: u8 = Cmd::Shl as u8;
    pub const SHR: u8 = Cmd::Shr as u8;
    pub const BIT_OR: u8 = Cmd::BitOr as u8;
    pub const BIT_AND: u8 = Cmd::BitAnd as u8;
    pub const XOR: u8 = Cmd::Xor as u8;
    // control flow
    pub const RETC: u8 = Cmd::Retc as u8;
    pub const RETV: u8 = Cmd::Retv as u8;
    pub const POPRET: u8 = Cmd::PopRet as u8;
    pub const JMP: u8 = Cmd::Jmp as u8;
    // memory
    pub const MOVC: u8 = Cmd::Movc as u8;
    pub const MOV: u8 = Cmd::Mov as u8;
    pub const NEWV: u8 = Cmd::Newv as u8;
    pub const NEWP: u8 = Cmd::Newp as u8;
    pub const DELV: u8 = Cmd::Delv as u8;
    pub const DELP: u8 = Cmd::Delp as u8;
    // miscellaneous
    pub const NOP: u8 = Cmd::Nop as u8;
    // functions
    pub const CALL: u8 = Cmd::Call as u8;
    // operand stack
    pub const PUSHVAR: u8 = Cmd::PushVar as u8;
    pub const POPVAR: u8 = Cmd::PopVar as u8;
    pub const STOREVAR: u8 = Cmd::StoreVar as u8;
    pub const DUP: u8 = Cmd::Dup as u8;
    pub const LDC: u8 = Cmd::Ldc as u8;
    pub const POP: u8 = Cmd::Pop as u8;
    // pub const LABEL: u8 = Cmd::Label as u8;
    // pub const JMP: u8 = Cmd::Jmp as u8;
    pub const CONV_TOP: u8 = Cmd::ConvTop as u8;
}