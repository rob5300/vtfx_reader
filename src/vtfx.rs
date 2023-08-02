const VTF_X360_MAJOR_VERSION: i32 = 0x0360;
const VTF_X360_MINOR_VERSION: i32 = 8;

pub const VTF_LEGACY_RSRC_IMAGE: [u8;4] = [0x30, 0, 0, 0];
pub const VTF_LEGACY_RSRC_LOW_RES_IMAGE: [u8;4] = [0x01, 0, 0, 0];

use std::{error::Error, mem, io, fmt};

use num_enum::TryFromPrimitive;

use crate::{ImageFormat, resource_entry_info::ResourceEntryInfo};

const RESOURCE_START: usize = 60;

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
    pub fn from(buffer: &[u8]) -> Result<VTFXHEADER, Box<dyn Error>>
    {
        let type_str_range = &buffer[0..4];
        let type_str = std::str::from_utf8(&type_str_range)?;
        if type_str != "VTFX"
        {
            let err = io::Error::new(io::ErrorKind::Other, "File is not VTFX file!");
            return Err(Box::new(err));
        }

        let mut vtfx: VTFXHEADER = { Default::default() };

        let mut i = 0;
        vtfx.file_type_string = String::from(type_str);
        i += 4;
        
        let mut version: [i32; 2] = [0; 2];
        for j in 0..2 {
            let start = i + (j * 4);
            let end = start + 4;
            let slice: &[u8] = &buffer[start..end];
            let value: i32 = i32::from_be_bytes(slice.try_into().unwrap());
            version[j] = value;
        }
        i += 4 * 2;
        vtfx.version = version;

        vtfx.header_size = i32::from_be_bytes(buffer[i..i+4].try_into().unwrap());
        i += 4;

        vtfx.flags = u32::from_be_bytes(buffer[i..i+4].try_into().unwrap());
        i += 4;

        vtfx.width = u16::from_be_bytes(buffer[i..i+2].try_into().unwrap());
        i += 2;

        vtfx.height = u16::from_be_bytes(buffer[i..i+2].try_into().unwrap());
        i += 2;

        vtfx.depth = u16::from_be_bytes(buffer[i..i+2].try_into().unwrap());
        i += 2;

        vtfx.num_frames = u16::from_be_bytes(buffer[i..i+2].try_into().unwrap());
        i += 2;

        vtfx.preload_data_size = u16::from_be_bytes(buffer[i..i+2].try_into().unwrap());
        i += 2;

        vtfx.mip_skip_count = u8::from_be_bytes(buffer[i..i+1].try_into().unwrap());
        i += 1;

        vtfx.num_resources = u8::from_be_bytes(buffer[i..i+1].try_into().unwrap());
        i += 1;

        //vtfx.reflectivity
        i += mem::size_of::<Vector>();

        vtfx.bump_scale = f32::from_be_bytes(buffer[i..i+4].try_into().unwrap());
        i += 4;

        let image_format_i32 = i32::from_be_bytes(buffer[i..i+4].try_into().unwrap());
        vtfx.image_format = ImageFormat::try_from_primitive(image_format_i32).unwrap();
        i += 4;

        //vtfx.low_res_image_sample
        i += 4;

        vtfx.compressed_size = u32::from_be_bytes(buffer[i..i+4].try_into().unwrap());
        i += 4;

        if cfg!(debug_assertions){ println!("[Debug] VTFX READ END: Current read position: {}, Data left: {} bytes.\n", i, buffer.len() - i); }

        Ok(vtfx)
    }

    pub fn get_resource_entry_infos(&self, buffer: &[u8]) -> Vec<ResourceEntryInfo>
    {
        let mut resource_entry_infos: Vec<ResourceEntryInfo> = Vec::new();
        let mut i = RESOURCE_START;
        for _res_num in 0..self.num_resources
        {
            let mut resource_entry_info: ResourceEntryInfo = { Default::default() };
            for x in 0..3
            {
                resource_entry_info.chTypeBytes[x] = buffer[i+x];
            }
            i += 4;
            resource_entry_info.resData = u32::from_be_bytes(buffer[i..i+4].try_into().unwrap());
            i += 4;
            resource_entry_infos.push(resource_entry_info);
        }

        resource_entry_infos
    }

    pub fn get_channels(&self) -> u16
    {
        if self.has_alpha()
        {
            return 4;
        }

        return 3;
    }

    pub fn has_alpha(&self) -> bool
    {
        (self.flags & 0x2000) != 0
    }

    pub fn has_onebit_alpha(&self) -> bool
    {
        (self.flags & 0x1000) != 0
    }

    pub fn all_mips(&self) -> bool
    {
        (self.flags & 0x00000400) != 0
    }

    pub fn no_mips(&self) -> bool
    {
        (self.flags & 0x00000100) != 0
    }

    pub fn hint_dx5(&self) -> bool
    {
        (self.flags & 0x0020) != 0
    }

    /// Get start of largest mip (width / 2, height / 2)
    pub fn get_mip0_start(&self) -> usize
    {
        let mut lower_mip_sizes: usize = 0;

        let mut width: usize = (self.width >> 1) as usize;
        let mut height: usize = (self.height >> 1) as usize;

        while width > 0 || height > 0
        {
            lower_mip_sizes += (width * height) * (self.get_channels() * self.depth) as usize;
            width = width >> 1;
            height = height >> 1;
        }

        lower_mip_sizes
    }
    /*
    pub fn ComputeMipLevelSubRect( Rect_t *pSrcRect, int nMipLevel, Rect_t *pSubRect )
    {
        Assert( pSrcRect->x >= 0 && pSrcRect->y >= 0 && 
            (pSrcRect->x + pSrcRect->width <= m_nWidth) &&  
            (pSrcRect->y + pSrcRect->height <= m_nHeight) );
        
        if (nMipLevel == 0)
        {
            *pSubRect = *pSrcRect;
            return;
        }

        float flInvShrink = 1.0f / (float)(1 << nMipLevel);
        pSubRect->x = ( int )( pSrcRect->x * flInvShrink );
        pSubRect->y = ( int )( pSrcRect->y * flInvShrink );
        pSubRect->width = (int)ceil( (pSrcRect->x + pSrcRect->width) * flInvShrink ) - pSubRect->x;
        pSubRect->height = (int)ceil( (pSrcRect->y + pSrcRect->height) * flInvShrink ) - pSubRect->y;
    }
    */
}

impl fmt::Display for VTFXHEADER {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(Version: {}.{}, Header Size: {}, Width: {}, Height: {}, Depth: {}, Num Frames: {}, Preload data size: {}, Mip skip count: {}, Bump Scale: {}, Image Format: {:?}, Compressed Size: {} | All mip: {}, No mip: {})",
            self.version[0], self.version[1], self.header_size, self.width, self.height, self.depth, self.num_frames, self.preload_data_size, self.mip_skip_count, self.bump_scale, self.image_format, self.compressed_size, self.all_mips(), self.no_mips())
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Vector {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Rect
{
    x: f32,
    y: f32,
    width: f32,
    height: f32
}