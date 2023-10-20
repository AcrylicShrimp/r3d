use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    pmx_header::{PmxConfig, PmxIndexSize},
    pmx_primitives::PmxVertexIndex,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmxSurfaceParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("failed to parse a PMX primitive: {0}")]
    PmxPrimitiveParseError(#[from] crate::pmx_primitives::PmxPrimitiveParseError),
    #[error("surface count `{count}` is invalid; it must be a multiple of 3")]
    InvalidSurfaceCount { count: usize },
}

impl ParseError for PmxSurfaceParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct PmxSurface {
    /// vertex indices in CW order (DirectX style)
    pub vertex_indices: [PmxVertexIndex; 3],
}

impl Parse for Vec<PmxSurface> {
    type Error = PmxSurfaceParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // surface count (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        // surface count is vertex count, not actual surface count in PMX
        let count = u32::parse(config, cursor)? as usize;

        // since all surfaces are triangles, surface count must be a multiple of 3
        if count % 3 != 0 {
            return Err(PmxSurfaceParseError::InvalidSurfaceCount { count });
        }

        // surface data (count * vertex_index_size bytes)
        let size = count * config.vertex_index_size.size();
        cursor.ensure_bytes::<Self::Error>(size)?;

        let count = count / 3;
        let mut surfaces = Vec::with_capacity(count);

        match config.vertex_index_size {
            PmxIndexSize::U8 => {
                let bytes = cursor.read_dynamic::<Self::Error>(count * 3 * 1)?;

                for index in 0..count {
                    let bytes = &bytes[index * 3 * 1..(index + 1) * 3 * 1];

                    let vertex_index_1 = PmxVertexIndex::new(u8::from_le_bytes(
                        bytes[0..1].try_into().unwrap(),
                    ) as u32);
                    let vertex_index_2 = PmxVertexIndex::new(u8::from_le_bytes(
                        bytes[1..2].try_into().unwrap(),
                    ) as u32);
                    let vertex_index_3 = PmxVertexIndex::new(u8::from_le_bytes(
                        bytes[2..3].try_into().unwrap(),
                    ) as u32);

                    let surface = PmxSurface {
                        vertex_indices: [vertex_index_1, vertex_index_2, vertex_index_3],
                    };

                    surfaces.push(surface);
                }
            }
            PmxIndexSize::U16 => {
                let bytes = cursor.read_dynamic::<Self::Error>(count * 3 * 2)?;

                for index in 0..count {
                    let bytes = &bytes[index * 3 * 2..(index + 1) * 3 * 2];

                    let vertex_index_1 = PmxVertexIndex::new(u16::from_le_bytes(
                        bytes[0..2].try_into().unwrap(),
                    ) as u32);
                    let vertex_index_2 = PmxVertexIndex::new(u16::from_le_bytes(
                        bytes[2..4].try_into().unwrap(),
                    ) as u32);
                    let vertex_index_3 = PmxVertexIndex::new(u16::from_le_bytes(
                        bytes[4..6].try_into().unwrap(),
                    ) as u32);

                    let surface = PmxSurface {
                        vertex_indices: [vertex_index_1, vertex_index_2, vertex_index_3],
                    };

                    surfaces.push(surface);
                }
            }
            PmxIndexSize::U32 => {
                let bytes = cursor.read_dynamic::<Self::Error>(count * 3 * 4)?;

                for index in 0..count {
                    let bytes = &bytes[index * 3 * 4..(index + 1) * 3 * 4];

                    let vertex_index_1 =
                        PmxVertexIndex::new(u32::from_le_bytes(bytes[0..4].try_into().unwrap()));
                    let vertex_index_2 =
                        PmxVertexIndex::new(u32::from_le_bytes(bytes[4..8].try_into().unwrap()));
                    let vertex_index_3 =
                        PmxVertexIndex::new(u32::from_le_bytes(bytes[8..12].try_into().unwrap()));

                    let surface = PmxSurface {
                        vertex_indices: [vertex_index_1, vertex_index_2, vertex_index_3],
                    };

                    surfaces.push(surface);
                }
            }
        }

        Ok(surfaces)
    }
}
