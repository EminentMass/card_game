pub struct TileWorld {
    pub chunks: Vec<TileChunk>,
}

pub type TileChunk = TileChunkGeneric<16, Tile>;

pub struct TileChunkGeneric<const L: usize, T> {
    pub tiles: [[[T; L]; L]; L], // 3D chunk of tiles. flattened length is  L^3
}

pub struct Tile {
    pub id: TileId,
    pub temperature: f32,
}

pub type TileId = u32;

/* pub fn get_tile_texture(tile: &Tile) -> TextureId {

} */
