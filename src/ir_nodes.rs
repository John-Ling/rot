pub enum IRNode {
    LEFT(IRLeft),
    RIGHT(IRRight),
    INCREMENT(IRIncrement),
    DECREMENT(IRDecrement),
    OUTPUT(IROutput),
    INPUT(IRInput),
    JUMPFWD(IRJumpFwd),
    JUMPBCK(IRJumpBck),
    CLEANUP(IRCleanup),
}

pub struct IRLeft {
    pub distance: usize,
}

pub struct IRRight {
    pub distance: usize,
}

pub struct IRIncrement {
    pub amount: usize,
}

pub struct IRDecrement {
    pub amount: usize,
}

pub struct IROutput {}

pub struct IRInput {}

pub struct IRJumpFwd {
    pub path_name: String,
}

pub struct IRJumpBck {
    pub path_name: String,
}

pub struct IRCleanup {}
