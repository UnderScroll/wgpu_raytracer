@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 1.0);
}

@group(0) @binding(0)
var texture: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    var frag_color = textureLoad(texture, vec2<i32>(position.xy), 0);
    frag_color = exp(log(frag_color) * 2.2);
    return frag_color;
}