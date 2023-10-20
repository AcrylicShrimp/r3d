use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    pmx_header::PmxConfig,
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

impl Parse for PmxSurface {
    type Error = PmxSurfaceParseError;

    fn parse(config: &PmxConfig, cursor: &mut impl Cursor) -> Result<Self, Self::Error> {
        // since surface has a fixed size, we don't need to check the size here
        let vertex_index_1 = PmxVertexIndex::parse(config, cursor)?;
        let vertex_index_2 = PmxVertexIndex::parse(config, cursor)?;
        let vertex_index_3 = PmxVertexIndex::parse(config, cursor)?;

        Ok(Self {
            vertex_indices: [vertex_index_1, vertex_index_2, vertex_index_3],
        })
    }
}

impl Parse for Vec<PmxSurface> {
    type Error = PmxSurfaceParseError;

    fn parse(config: &PmxConfig, cursor: &mut impl Cursor) -> Result<Self, Self::Error> {
        // surface count (4 bytes)
        let size = 4;
        cursor.checked().ensure_bytes::<Self::Error>(size)?;

        // surface count is vertex count, not actual surface count in PMX
        let count = u32::parse(config, cursor)? as usize;

        // since all surfaces are triangles, surface count must be a multiple of 3
        if count % 3 != 0 {
            return Err(PmxSurfaceParseError::InvalidSurfaceCount { count });
        }

        // surface data (count * vertex_index_size bytes)
        let size = count * config.vertex_index_size.size();
        cursor.checked().ensure_bytes::<Self::Error>(size)?;

        let count = count / 3;
        let mut surfaces = Vec::with_capacity(count);

        for _ in 0..count {
            let surface = PmxSurface::parse(config, cursor)?;
            surfaces.push(surface);
        }

        Ok(surfaces)
    }
}
