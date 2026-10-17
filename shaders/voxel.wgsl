struct VoxelInstance {
    @location(0) position: vec3<f32>,
    @location(1) size: f32,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    voxel: VoxelInstance
) -> VertexOutput {
    var cube_vertices = array<vec3<f32>, 8>(
        vec3<f32>(-0.5, -0.5,  0.5),
        vec3<f32>( 0.5, -0.5,  0.5),
        vec3<f32>( 0.5,  0.5,  0.5),
        vec3<f32>(-0.5,  0.5,  0.5),
        vec3<f32>(-0.5, -0.5, -0.5),
        vec3<f32>( 0.5, -0.5, -0.5),
        vec3<f32>( 0.5,  0.5, -0.5),
        vec3<f32>(-0.5,  0.5, -0.5)
    );
    var cube_indices = array<u32, 36>(
        0, 1, 2,  2, 3, 0,
        1, 5, 6,  6, 2, 1,
        5, 4, 7,  7, 6, 5,
        4, 0, 3,  3, 7, 4,
        3, 2, 6,  6, 7, 3,
        4, 5, 1,  1, 0, 4
    );

    let vertex_index = cube_indices[in_vertex_index];
    let local_pos = cube_vertices[vertex_index];

    let world_pos = local_pos * voxel.size + voxel.position;

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.color = voxel.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}