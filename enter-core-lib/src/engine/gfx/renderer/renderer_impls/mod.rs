use super::{GenericBufferAllocation, HostBuffer, Renderer};
use crate::engine::gfx::{semantic_inputs, Color, SemanticShaderInputKey, SpriteHandle};
use zerocopy::AsBytes;

mod mesh_renderer;

pub use mesh_renderer::*;

// pub struct SpriteRendererData {
//     pub sprite: SpriteHandle,
//     pub color: Color,
// }

// pub struct SpriteRenderer {}

// impl Renderer for SpriteRenderer {
//     type RenderData = SpriteRendererData;

//     fn vertex_count(&self) -> u32 {
//         6
//     }

//     fn copy_semantic_per_instance_input(
//         &self,
//         key: SemanticShaderInputKey,
//         data: &Self::RenderData,
//         allocation: &mut GenericBufferAllocation<HostBuffer>,
//     ) {
//         match key {
//             semantic_inputs::KEY_TRANSFORM_ROW_0 => {
//                 let row = [1f32, 0f32, 0f32];
//                 allocation.with_data_mut(|data| data.copy_from_slice(row.as_bytes()));
//             }
//             semantic_inputs::KEY_TRANSFORM_ROW_1 => {
//                 let row = [0f32, 1f32, 0f32];
//                 allocation.with_data_mut(|data| data.copy_from_slice(row.as_bytes()));
//             }
//             semantic_inputs::KEY_TRANSFORM_ROW_2 => {
//                 let row = [0f32, 0f32, 1f32];
//                 allocation.with_data_mut(|data| data.copy_from_slice(row.as_bytes()));
//             }
//             semantic_inputs::KEY_SPRITE_SIZE => {
//                 let size = [data.sprite.width(), data.sprite.height()];
//                 allocation.with_data_mut(|data| data.copy_from_slice(size.as_bytes()));
//             }
//             semantic_inputs::KEY_SPRITE_COLOR => {
//                 let color = [data.color.r, data.color.g, data.color.b, data.color.a];
//                 allocation.with_data_mut(|data| data.copy_from_slice(color.as_bytes()));
//             }
//             _ => {}
//         }
//     }

//     fn copy_semantic_per_vertex_input(
//         &self,
//         key: SemanticShaderInputKey,
//         data: &Self::RenderData,
//         vertex_index: u32,
//         allocation: &mut GenericBufferAllocation<HostBuffer>,
//     ) {
//         match key {
//             semantic_inputs::KEY_POSITION => {
//                 let x = [0f32, 0f32, 1f32, 1f32, 0f32, 1f32][vertex_index as usize];
//                 let y = [1f32, 0f32, 1f32, 1f32, 0f32, 0f32][vertex_index as usize];
//                 allocation.with_data_mut(|data| data.copy_from_slice([x, y].as_bytes()));
//             }
//             semantic_inputs::KEY_UV => {
//                 let mapping = data.sprite.mapping();
//                 let inv_width = (data.sprite.texture().width as f32).recip();
//                 let inv_height = (data.sprite.texture().height as f32).recip();
//                 let rect = [
//                     mapping.x_min as f32 * inv_width,
//                     mapping.y_min as f32 * inv_height,
//                     mapping.x_max as f32 * inv_width,
//                     mapping.y_max as f32 * inv_height,
//                 ];
//                 let x =
//                     [rect[0], rect[0], rect[1], rect[1], rect[0], rect[1]][vertex_index as usize];
//                 let y =
//                     [rect[3], rect[2], rect[3], rect[3], rect[2], rect[2]][vertex_index as usize];
//                 allocation.with_data_mut(|data| data.copy_from_slice([x, y].as_bytes()));
//             }
//             _ => {}
//         }
//     }
// }
