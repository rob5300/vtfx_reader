const VTF_X360_MAJOR_VERSION: i32 = 0x0360;
const VTF_X360_MINOR_VERSION: i32 = 8;

pub const VTF_LEGACY_RSRC_IMAGE: [u8;4] = [0x30, 0, 0, 0];
pub const VTF_LEGACY_RSRC_LOW_RES_IMAGE: [u8;4] = [0x01, 0, 0, 0];

use crate::ImageFormat;

#[repr(C)]
#[derive(Debug, Default)]
//https://developer.valvesoftware.com/wiki/VTFX_file_format
//https://github.com/ValveSoftware/source-sdk-2013/blob/master/sp/src/public/vtf/vtf.h#L551
pub struct VTFXHEADER {
    pub file_type_string: String,               // VTFX.
    pub version: [i32; 2],                     // version[0].version[1].
    pub header_size: i32,
    pub flags: u32,
    pub width: u16,                    // actual width of data in file.
    pub height: u16,                   // actual height of data in file.
    pub depth: u16,                    // actual depth of data in file.
    pub num_frames: u16,
    pub preload_data_size: u16,         // exact size of preload data (may extend into image!).
    pub mip_skip_count: u8,             // used to resconstruct mapping dimensions.
    pub num_resources: u8,
    pub reflectivity: Vector,           // Resides on 16 byte boundary!.
    pub bump_scale: f32,
    pub image_format: ImageFormat,
    pub low_res_image_sample: [u8; 4],
    pub compressed_size: u32,
}

impl VTFXHEADER
{
    pub fn has_alpha(&self) -> bool
    {
        (self.flags & 0x00002000) != 0
    }
}

#[derive(Debug, Default)]
pub struct Vector {
    x: f32,
    y: f32,
    z: f32,
}
