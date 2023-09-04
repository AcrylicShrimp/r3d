// use crate::engine::gfx::Color;
// use codegen::{
//     hidden, lua_user_data_method, ops_to_string, LuaEnum, LuaError, LuaUserData, LuaUserDataArc,
// };
// use parking_lot::Mutex;
// use std::{
//     collections::{hash_map::Entry, HashMap},
//     fmt::Display,
//     num::NonZeroU32,
// };
// use thiserror::Error;

// pub struct Map {
//     pub width: u32,
//     pub height: u32,
//     pub tiles: Vec<Tile>,
// }

// impl Map {
//     pub fn new(width: u32, height: u32, tiles: impl Into<Vec<Tile>>) -> Self {
//         Self {
//             width,
//             height,
//             tiles: tiles.into(),
//         }
//     }
// }

// #[derive(LuaUserData, Clone)]
// pub struct TileObject {
//     pub tile: LuaTile,
// }

// #[lua_user_data_method]
// impl TileObject {
//     pub fn new(tile: LuaTile) -> Self {
//         Self { tile }
//     }
// }

// #[derive(LuaUserData, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct TileId(NonZeroU32);

// #[lua_user_data_method]
// #[ops_to_string]
// impl TileId {
//     #[hidden]
//     pub fn new(id: u32) -> Option<Self> {
//         NonZeroU32::new(id).map(Self)
//     }

//     pub fn get(self) -> u32 {
//         self.0.get()
//     }

//     #[hidden]
//     pub fn get_usize(self) -> usize {
//         self.0.get() as usize
//     }
// }

// impl Display for TileId {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.0.fmt(f)
//     }
// }

// #[derive(Debug, Clone)]
// pub struct Tile {
//     // #[readonly]
//     pub id: TileId,
//     // #[readonly]
//     pub kind: TileKind,
//     // #[readonly]
//     pub glyph: char,
//     // #[readonly]
//     pub fore_color: Color,
//     // #[readonly]
//     pub back_color: Color,
// }

// // #[lua_user_data_method]
// impl Tile {
//     #[hidden]
//     pub fn new(
//         id: TileId,
//         kind: TileKind,
//         glyph: char,
//         fore_color: Color,
//         back_color: Color,
//     ) -> Self {
//         Self {
//             id,
//             kind,
//             glyph,
//             fore_color,
//             back_color,
//         }
//     }
// }

// #[derive(LuaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum TileKind {
//     Wall,
//     Floor,
// }

// #[derive(LuaError, Error, Debug)]
// pub enum TileKindParseError {
//     #[error("invalid tile kind: {0}")]
//     InvalidTileKind(String),
// }

// impl TryFrom<String> for TileKind {
//     type Error = TileKindParseError;

//     fn try_from(value: String) -> Result<Self, Self::Error> {
//         match value.to_lowercase().as_str() {
//             "wall" => Ok(Self::Wall),
//             "floor" => Ok(Self::Floor),
//             _ => Err(TileKindParseError::InvalidTileKind(value)),
//         }
//     }
// }

// impl From<TileKind> for String {
//     fn from(value: TileKind) -> Self {
//         match value {
//             TileKind::Wall => "wall".to_owned(),
//             TileKind::Floor => "floor".to_owned(),
//         }
//     }
// }

// #[derive(LuaUserDataArc)]
// pub struct TileManager {
//     #[hidden]
//     tiles: Mutex<HashMap<TileId, LuaTile>>,
// }

// #[lua_user_data_method]
// impl TileManager {
//     #[hidden]
//     pub fn new() -> Self {
//         Self {
//             tiles: Mutex::new(HashMap::new()),
//         }
//     }

//     pub fn register_tile(&self, tile: LuaTile) -> Result<LuaTile, TileRegistrationError> {
//         match self.tiles.lock().entry(tile.id) {
//             Entry::Occupied(_) => {
//                 return Err(TileRegistrationError::TileIdAlreadyRegistered(tile.id))
//             }
//             Entry::Vacant(entry) => {
//                 entry.insert(tile.clone());
//             }
//         }

//         Ok(tile)
//     }

//     pub fn get_tile(&self, tile_id: TileId) -> Option<LuaTile> {
//         self.tiles.lock().get(&tile_id).cloned()
//     }

//     pub fn instantiate_tile_object(&self, tile_id: TileId) -> Option<TileObject> {
//         self.get_tile(tile_id).map(|tile| TileObject { tile })
//     }
// }

// #[derive(LuaError, Error, Debug)]
// pub enum TileRegistrationError {
//     #[error("tile id {0} is already registered")]
//     TileIdAlreadyRegistered(TileId),
// }
