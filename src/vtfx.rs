use num_enum::TryFromPrimitive;

const VTF_X360_MAJOR_VERSION: i32 = 0x0360;
const VTF_X360_MINOR_VERSION: i32 = 8;

pub const VTF_LEGACY_RSRC_IMAGE: [u8;4] = [0x30, 0, 0, 0];

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
    // *** followed by *** ResourceEntryInfo resources[0];
}

#[derive(Debug, Default)]
pub struct Vector {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Default, PartialEq, TryFromPrimitive)]
#[repr(i32)]
#[allow(non_camel_case_types)] //Keep enums same as source
//https://github.com/ValveSoftware/source-sdk-2013/blob/master/sp/src/public/bitmap/imageformat.h#L35
pub enum ImageFormat 
{
    #[default]
	IMAGE_FORMAT_UNKNOWN = -1,
	IMAGE_FORMAT_RGBA8888 = 0, 
	IMAGE_FORMAT_ABGR8888, 
	IMAGE_FORMAT_RGB888, 
	IMAGE_FORMAT_BGR888,
	IMAGE_FORMAT_RGB565, 
	IMAGE_FORMAT_I8,
	IMAGE_FORMAT_IA88,
	IMAGE_FORMAT_P8,
	IMAGE_FORMAT_A8,
	IMAGE_FORMAT_RGB888_BLUESCREEN,
	IMAGE_FORMAT_BGR888_BLUESCREEN,
	IMAGE_FORMAT_ARGB8888,
	IMAGE_FORMAT_BGRA8888,
	IMAGE_FORMAT_DXT1,
	IMAGE_FORMAT_DXT3,
	IMAGE_FORMAT_DXT5,
	IMAGE_FORMAT_BGRX8888,
	IMAGE_FORMAT_BGR565,
	IMAGE_FORMAT_BGRX5551,
	IMAGE_FORMAT_BGRA4444,
	IMAGE_FORMAT_DXT1_ONEBITALPHA,
	IMAGE_FORMAT_BGRA5551,
	IMAGE_FORMAT_UV88,
	IMAGE_FORMAT_UVWQ8888,
	IMAGE_FORMAT_RGBA16161616F,
	IMAGE_FORMAT_RGBA16161616,
	IMAGE_FORMAT_UVLX8888,
	IMAGE_FORMAT_R32F,			// Single-channel 32-bit floating point
	IMAGE_FORMAT_RGB323232F,
	IMAGE_FORMAT_RGBA32323232F,

	// Depth-stencil texture formats for shadow depth mapping
	IMAGE_FORMAT_NV_DST16,		// 
	IMAGE_FORMAT_NV_DST24,		//
	IMAGE_FORMAT_NV_INTZ,		// Vendor-specific depth-stencil texture
	IMAGE_FORMAT_NV_RAWZ,		// formats for shadow depth mapping 
	IMAGE_FORMAT_ATI_DST16,		// 
	IMAGE_FORMAT_ATI_DST24,		//
	IMAGE_FORMAT_NV_NULL,		// Dummy format which takes no video memory

	// Compressed normal map formats
	IMAGE_FORMAT_ATI2N,			// One-surface ATI2N / DXN format
	IMAGE_FORMAT_ATI1N,			// Two-surface ATI1N format

	// Depth-stencil texture formats
	IMAGE_FORMAT_X360_DST16,
	IMAGE_FORMAT_X360_DST24,
	IMAGE_FORMAT_X360_DST24F,
	// supporting these specific formats as non-tiled for procedural cpu access
	IMAGE_FORMAT_LINEAR_BGRX8888,
	IMAGE_FORMAT_LINEAR_RGBA8888,
	IMAGE_FORMAT_LINEAR_ABGR8888,
	IMAGE_FORMAT_LINEAR_ARGB8888,
	IMAGE_FORMAT_LINEAR_BGRA8888,
	IMAGE_FORMAT_LINEAR_RGB888,
	IMAGE_FORMAT_LINEAR_BGR888,
	IMAGE_FORMAT_LINEAR_BGRX5551,
	IMAGE_FORMAT_LINEAR_I8,
	IMAGE_FORMAT_LINEAR_RGBA16161616,

	IMAGE_FORMAT_LE_BGRX8888,
	IMAGE_FORMAT_LE_BGRA8888,

	NUM_IMAGE_FORMATS
}

#[repr(C)]
#[derive(Debug, Default)]
#[allow(non_snake_case)] 
pub struct ResourceEntryInfo
{
	pub chTypeBytes: [u8; 4],
	pub resData: u32	// Resource data or offset from the beginning of the file
}
