use std::collections::HashMap;
use once_cell::sync::Lazy;

use num_enum::TryFromPrimitive;

#[derive(Debug, Default, PartialEq, TryFromPrimitive, Eq, Hash)]
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

#[derive(Clone)]
pub struct image_format_info
{
    pub channels: u16,
    pub depth: u16,
    pub channel_order: Vec<u16>,
    pub bc_format: Option<texpresso::Format>
}

impl image_format_info
{
    fn new(channels: u16, depth: u16, channel_order: Vec<u16>) -> image_format_info
    {
        image_format_info {
            channels: channels,
            depth: depth,
            channel_order: channel_order,
            bc_format: None
        }
    }

    fn new_with_bc(channels: u16, depth: u16, channel_order: Vec<u16>, bc_format: Option<texpresso::Format>) -> image_format_info
    {
        image_format_info {
            channels: channels,
            depth: depth,
            channel_order: channel_order,
            bc_format: bc_format
        }
    }
}

static IMAGE_FORMAT_INFO_MAP: Lazy<HashMap<ImageFormat, image_format_info>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(ImageFormat::IMAGE_FORMAT_DXT1, image_format_info::new_with_bc(3, 1, vec![0,1,2], Option::from(texpresso::Format::Bc1)));
    map.insert(ImageFormat::IMAGE_FORMAT_DXT5, image_format_info::new_with_bc(4, 1, vec![0,1,2,3], Option::from(texpresso::Format::Bc3)));
    map.insert(ImageFormat::IMAGE_FORMAT_RGBA16161616, image_format_info::new_with_bc(4, 2, vec![0,1,2,3], Option::None));
    return map;
});

impl ImageFormat
{
    pub fn get_format_info(&self) -> Option<&image_format_info>
    {
        return IMAGE_FORMAT_INFO_MAP.get(&self);
    }
}
