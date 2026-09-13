use voxt_core::prelude::{BatWorld, WorldPos};

#[derive(Debug, PartialEq, Eq)]
pub enum WorldCommand {
    Request(WorldRequest),
    Query(WorldQuery),
}

impl From<WorldRequest> for WorldCommand {
    fn from(value: WorldRequest) -> Self {
        Self::Request(value)
    }
}

impl From<WorldQuery> for WorldCommand {
    fn from(value: WorldQuery) -> Self {
        Self::Query(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum WorldRequest {
    SetBlock(BatWorld),
    SetBlockBulk(Box<[BatWorld]>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum WorldQuery {
    GetBlock(WorldPos),
    GetBlockBulk(Box<[WorldPos]>),
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
