use std::mem::transmute;

fn q_rsqrt(number: f32) -> f32 {
	let mut i: i32;
	let mut x2: f32;
	let mut y: f32;
	const THREEHALFS: f32 = 1.5;

	x2 = number * 0.5;
	y = number;
	// evil floating point bit level hacking
	unsafe {
		i = transmute::<f32, i32>(y);
	}
	// what the fuck?
	i = 0x5f3759df - (i >> 1);
	unsafe {
		y = transmute::<i32, f32>(i);
	}
	// 1st iteration
	y = y * (THREEHALFS - (x2 * y * y));

	y
}
