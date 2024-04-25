
@group(0) @binding(0) var<uniform> screen_size: vec2<f32>;
@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(2) @binding(0) var sprite_sampler: sampler;

struct InstanceInput {
  @location(0) transform_row_0: vec4<f32>,
  @location(1) transform_row_1: vec4<f32>,
  @location(2) transform_row_2: vec4<f32>,
  @location(3) transform_row_3: vec4<f32>,
  @location(4) sprite_size: vec2<f32>,
  @location(5) sprite_offset: vec2<f32>,
  @location(6) sprite_uv_min: vec2<f32>,
  @location(7) sprite_uv_max: vec2<f32>,
  @location(8) sprite_color: vec4<f32>,
};

struct VertexInput {
  @location(9) position: vec3<f32>,
};

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec4<f32>,
  @location(1) uv: vec2<f32>,
};

struct FragmentOutput {
  @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(instance: InstanceInput, vertex: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  let transform = mat4x4<f32>(instance.transform_row_0, instance.transform_row_1, instance.transform_row_2, instance.transform_row_3);
  out.position = (transform * (vec4<f32>(instance.sprite_offset, 0.0, 0.0) + vec4<f32>(instance.sprite_size, 1.0, 1.0) * vec4<f32>(vertex.position, 1.0))) / vec4<f32>(screen_size * 0.5, 1.0, 1.0);
  out.color = instance.sprite_color;
  out.uv = instance.sprite_uv_min + (instance.sprite_uv_max - instance.sprite_uv_min) * vertex.position.xy;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
  var out: FragmentOutput;
  out.color = in.color * textureSample(sprite_texture, sprite_sampler, in.uv);
  return out;
}
