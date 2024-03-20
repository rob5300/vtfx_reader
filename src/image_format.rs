use std::error::Error;
use std::{collections::HashMap};
use std::io;
use once_cell::sync::Lazy;

use num_enum::TryFromPrimitive;

#[derive(Debug, Default, PartialEq, TryFromPrimitive, Eq, Hash, Copy, Clone)]
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
    ///Number of colour channels
    pub channels: u16,
    ///Depth of colours in bytes
    pub depth: u16,
    ///Order of channels in relation to RGBA (0,1,2,3)
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

    pub fn try_get_bc_format(&self) -> Result<texpresso::Format, Box<dyn Error>>
    {
        let bc_format = self.bc_format.ok_or("Image format is not DXT")?;
        Ok(bc_format)
    }
}

///Correct endianness of dxt bc data
pub fn correct_dxt_endianness(format: &texpresso::Format, data: &mut [u8]) -> Result<(), Box<dyn Error>>
{
    //https://learn.microsoft.com/en-us/windows/win32/direct3d10/d3d10-graphics-programming-guide-resources-block-compression#bc1

    ///Fix single block of bc1 data
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

        //bitmap u32 fix (treat as u16[2])
        for i in 0..2
        {
            let index = bitmap_index + (i * 2);
            let bitmap: u16 = u16::from_be_bytes(block[index..index + 2].try_into()?);
            let bitmap_bytes = bitmap.to_le_bytes();
            for x in 0..2
            {
                block[index + x] = bitmap_bytes[x];
            }
        }
        
        //Reverse bitmap indexes
        /*
        for i in 0..4
        {
            let byte = &block[bitmap_index + i];
            let mut new_byte = 0u8;
            new_byte |= (0b00000011 & byte) << 6;
            new_byte |= (0b00001100 & byte) << 2;
            new_byte |= (0b00110000 & byte) >> 2;
            new_byte |= (0b11000000 & byte) >> 6;
            block[bitmap_index + i] = new_byte;
        }
        */

        Ok(())
    }

    match format {
        texpresso::Format::Bc3 => 
        {
            if data.len() % 16 != 0
            {
                let err = io::Error::new(io::ErrorKind::Other, format!("Length of dxt5 buffer should be multiple of 16. Length: {}", data.len()));
                return Err(Box::new(err));
            }

            for block in data.chunks_exact_mut(16)
            {
                //Use bc1 fix on the last part of the block
                fix_bc1(&mut block[8..16])?;
            }
        },
        texpresso::Format::Bc1 =>
        {
            if data.len() % 8 != 0
            {
                let err = io::Error::new(io::ErrorKind::Other, format!("Length of dxt1 buffer should be multiple of 8. Length: {}", data.len()));
                return Err(Box::new(err));
            }

            for block in data.chunks_exact_mut(8)
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

#[allow(non_snake_case)]
pub fn GetNumMipMapLevels(mut width: i32, mut height: i32, mut depth: i32) -> i32
{
	if depth <= 0
	{
		depth = 1;
	}

	if width < 1 || height < 1 || depth < 1
    {
        return 0;
    }

	let mut numMipLevels = 1;
	loop
	{
		if width == 1 && height == 1 && depth == 1
        {
            break;
        }

		width >>= 1;
		height >>= 1;
		depth >>= 1;
        
		if width < 1
		{
			width = 1;
		}

		if height < 1
		{
			height = 1;
		}

		if depth < 1
		{
			depth = 1;
		}
		numMipLevels += 1;
	}
	return numMipLevels;
}

pub fn GetMipMapLevelByteOffset(mut width: i32, mut height: i32, image_format: &image_format_info, mut skip_mip_levels: i32) -> usize
{
	let mut offset: usize = 0;

	while skip_mip_levels > 0
	{
		offset += (width * height * (image_format.depth * image_format.channels) as i32) as usize;
		if width == 1 && height == 1
		{
			break;
		}

		width >>= 1;
		height >>= 1;
		if width < 1 
		{
			width = 1;
		}
		if height < 1 
		{
			height = 1;
		}
		skip_mip_levels -= 1;
	}
	return offset;
}

static IMAGE_FORMAT_INFO_MAP: Lazy<HashMap<ImageFormat, image_format_info>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(ImageFormat::IMAGE_FORMAT_DXT1, image_format_info::new_with_bc(3, 1, vec![0,1,2], Option::from(texpresso::Format::Bc1)));
    map.insert(ImageFormat::IMAGE_FORMAT_DXT5, image_format_info::new_with_bc(4, 1, vec![0,1,2,3], Option::from(texpresso::Format::Bc3)));
    map.insert(ImageFormat::IMAGE_FORMAT_RGBA16161616, image_format_info::new(4, 2, vec![0,1,2,3]));
    map.insert(ImageFormat::IMAGE_FORMAT_BGRX8888, image_format_info::new(4, 1, vec![2,1,0,3]));
    map.insert(ImageFormat::IMAGE_FORMAT_LINEAR_BGRX8888, image_format_info::new(4, 1, vec![2,1,0,3]));
    map.insert(ImageFormat::IMAGE_FORMAT_RGBA8888, image_format_info::new(4, 1, vec![0,1,2,3]));
    map.insert(ImageFormat::IMAGE_FORMAT_ABGR8888, image_format_info::new(4, 1, vec![3,2,1,0]));
    map.insert(ImageFormat::IMAGE_FORMAT_RGB888, image_format_info::new(3, 1, vec![0,1,2]));
    map.insert(ImageFormat::IMAGE_FORMAT_BGR888, image_format_info::new(3, 1, vec![2,1,0]));
    map.insert(ImageFormat::IMAGE_FORMAT_ARGB8888, image_format_info::new(4, 1, vec![3,2,1,0]));
    map.insert(ImageFormat::IMAGE_FORMAT_BGRA8888, image_format_info::new(4, 1, vec![2,1,0,3]));
    return map;
});

impl ImageFormat
{
    pub fn get_format_info(&self) -> Option<&image_format_info>
    {
        let num = *self as i32;
        let format_info = IMAGE_FORMAT_INFO_MAP.get(&self);
        if format_info.is_some() && num >= 30
        {
            println!("!!Warning!! The image format '{:?}' is untested. If result is garbage please open an issue on github.", &self);
        }
        format_info
    }

    pub fn try_get_format_info(&self) -> Result<&image_format_info, Box<dyn Error>>
    {
        let image_format = self.get_format_info().ok_or("vtfx is an unsupported/unknown format")?;
        Ok(image_format)
    }
}
