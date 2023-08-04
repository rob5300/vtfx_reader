use std::error::Error;
use std::{collections::HashMap};
use std::io;
use once_cell::sync::Lazy;

use num_enum::TryFromPrimitive;

#[derive(Debug, Default, PartialEq, TryFromPrimitive, Eq, Hash)]
#[repr(i32)]
#[allow(non_camel_case_types, non_upper_case_globals)] //Keep enums same as source
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

///Correct endianness of dxt bc data
pub fn correct_dxt_endianness(format: &texpresso::Format, data: &mut [u8]) -> Result<(), Box<dyn Error>>
{
    //https://stackoverflow.com/questions/67066835/how-to-decompress-a-bc3-unorm-dds-texture-format
    /*
    struct BC1
    {
        uint16_t    rgb[2]; // 565 colors
        uint32_t    bitmap; // 2bpp rgb bitmap
    };

    struct BC3
    {
        uint8_t     alpha[2];   // alpha values
        uint8_t     bitmap[6];  // 3bpp alpha bitmap
        BC1         bc1;        // BC1 rgb data
    };
    */

    ///bc1 block fixer
    fn fix_bc1(block: &mut [u8]) -> Result<(), Box<dyn Error>>
    {
        let rgb_index: usize = 0;
        let bitmap_index = rgb_index + 4;
        
        //rgb data fix (u16[2]])
        for i in 0..2
        {
            let index = rgb_index + (2 * i);
            let u16 = u16::from_be_bytes(block[index..index + 2].try_into()?);
            let u16_bytes = u16.to_le_bytes();
            
            for j in 0..u16_bytes.len()
            {
                block[index + j] = u16_bytes[j];
            }
        }

        //bitmap u32 fix
        let bitmap: u32 = u32::from_be_bytes(block[bitmap_index..bitmap_index + 4].try_into()?);
        let bitmap_bytes = bitmap.to_le_bytes();
        for i in 0..4
        {
            block[bitmap_index + i] = bitmap_bytes[i];
        }

        Ok(())
    }

    match format {
        texpresso::Format::Bc3 => 
        {
            if data.len() % 16 != 0
            {
                let err = io::Error::new(io::ErrorKind::Other, format!("Length of dxt buffer should be multiple of 16. Length: {}", data.len()));
                return Err(Box::new(err));
            }

            for block in data.chunks_mut(16)
            {
                //Use bc1 fix on the last part of the block
                fix_bc1(&mut block[8..16])?;
            }
        },
        texpresso::Format::Bc1 =>
        {
            for block in data.chunks_mut(8)
            {
                fix_bc1(block)?;
            }
        }
        _ =>
        {
            let err = io::Error::new(io::ErrorKind::Other, format!("endianness fix not implemented for dxt bc format: {:?}", format));
            return Err(Box::new(err));
        }
    };

    Ok(())
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
