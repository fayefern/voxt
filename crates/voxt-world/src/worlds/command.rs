use voxt_core::prelude::{BlockId, WorldPos};

pub enum WorldCommand {
    GetBlock(WorldPos),
    SetBlock(WorldPos, BlockId),
    RemoveBlock(WorldPos),
    // Add more commands as needed.
}
