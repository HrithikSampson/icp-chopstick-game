
// fn get_random_values(bytes:&mut [u8; 16]) {
//     for (_i,num) in bytes.iter_mut().enumerate() {
//         let temp = 0;
//         let address_temp = &temp as *const i32;
//         let rng = address_temp as i32;
//         *num = ((rng % 256) + 256) as u8;

//     }
// }

// pub fn generate_uuid_v4() -> String {
//     let mut bytes = [0u8; 16];
//     // This will fill the bytes array with cryptographic random bytes
//     get_random_values(&mut bytes); 
//     // Set the version to 4 (Random) and the variant to 1
//     bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
//     bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 1
//     format!("{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
//             bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
//             bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15])
// }

// not working