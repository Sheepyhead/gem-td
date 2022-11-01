pub fn approx_equal(a: f32, b: f32) -> bool {
    let margin = f32::EPSILON;
    (a - b).abs() < margin
}
