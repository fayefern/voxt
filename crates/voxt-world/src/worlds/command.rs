use voxt_core::prelude::{BatWorld, WorldPos};

#[derive(Debug, PartialEq, Eq)]
pub enum WorldRequest<'a> {
    GetBlock(WorldPos),
    GetBlockBulk(&'a [WorldPos]),
    SetBlock(BatWorld),
    SetBlockBulk(&'a [BatWorld]),
}

#[derive(Debug, PartialEq, Eq)]
pub enum WorldResponse {
    Invalid { reason: InvalidReason },
    Failure { reason: FailureReason },
    ReturnBlock(BatWorld),
    ReturnBlockBulk(Box<[BatWorld]>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum InvalidReason {
    EmptyBlockBulkRequest,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FailureReason {
    ChunkNotInMemory,
}
