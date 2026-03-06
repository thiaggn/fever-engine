use glam::Mat4;

/// Cria uma matriz de projeção com perspectiva.
pub fn perspective(near: f32, far: f32, fov: f32, ratio: f32) -> Mat4 {
	let top = f32::tan(fov.to_radians()/2.0) * near;
	let right = top * ratio;
	
	#[rustfmt::skip]
	let matrix = [
		[near/right, 0.0,      0.0,             0.0                 ],
		[0.0,        near/top, 0.0,             0.0                 ],
		[0.0,        0.0,      -far/(far-near), -far*near/(far-near)],
		[0.0,        0.0,      -1.0,            0.0                 ],
	];
	
	return Mat4::from_cols_array_2d(&matrix).transpose();
}