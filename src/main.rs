use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use args::Args;
use clap::Parser;
use image::DynamicImage;
use image::GenericImage;
use image::Rgba;
use image_format::correct_dxt_endianness;
use once_cell::sync::Lazy;
use vtfx::VTFXHEADER;
use std::convert::TryInto;

use crate::image_format::ImageFormat;
use crate::resource_entry_info::ResourceEntryInfo;
use crate::vtfx::VTF_LEGACY_RSRC_IMAGE;

mod vtfx;
mod image_format;
mod resource_entry_info;
mod args;

const LZMA_MAGIC: &[u8;4] = b"LZMA";
static ARGS: Lazy<Args> = Lazy::new(|| { Args::parse() });
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("VTFX Reader {VERSION} [github.com/rob5300/vtfx_reader]");

    if !ARGS.input.exists() {
        println!("Error: No input file given. Run with --help to see arguments.");
        exit(1);
    }

    //Check path is file
    let path = Path::new(&ARGS.input);

    if !path.exists() {
        println!("Provided path '{}' does not exist!", ARGS.input.as_os_str().to_string_lossy());
        return;
    }

    if path.is_file()
    {
        println!("Opening '{}'...", path.to_string_lossy());
        match read_vtfx(&path) {
            Ok(_) => {println!("VTFX processing complete")},
            Err(e) => {println!("Failed to open file: {e}")},
        };
    }
    else if path.is_dir()
    {
        println!("Will open all vtf files in given folder");
        match read_all_vtfx_in_folder(&path) {
            Ok(_) => {},
            Err(e) => {println!("Failed to process all files in input folder: {e}")},
        }
    }
}

fn read_all_vtfx_in_folder(path: &Path) -> Result<(), Box<dyn Error>>
{
    for file in fs::read_dir(path)?
    {
        let path = file?.path();
        if path.is_dir()
        {
            read_all_vtfx_in_folder(path.as_path())?;
        }
        else
        {
            println!("Opening '{}'...", path.to_string_lossy());
            match read_vtfx(&path) {
                Ok(_) => {println!("VTFX processing complete")},
                Err(e) => {println!("Failed to open file: {e}")},
            };
        }
    }

    Ok(())
}

///Open a vtfx file at path and read its data
fn read_vtfx(path: &Path) -> Result<VTFXHEADER, Box<dyn Error>>
{
    //Validate file extension
    let extension = path.extension().unwrap_or_default().to_str().unwrap_or_default().to_lowercase();
    if extension != "vtf" && extension != "vtfx"
    {
        return Err(Box::new(io::Error::new(io::ErrorKind::InvalidInput, "File extension is not 'vtf' or 'vtfx")));
    }

    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let mut buffer: Vec<u8> = Vec::new();

    reader.read_to_end(&mut buffer)?;

    let vtfx = VTFXHEADER::from(&buffer)?;

    println!("    Vtfx is for {}", match vtfx.is_xbox() {
        true => "Xbox 360",
        false => "PS3/Other"
    });

    let dxt_hint = vtfx.hint_dx5();
    if cfg!(debug_assertions) && dxt_hint
    {
        println!("[Debug] Has dxt5 hint flag");
    }

    if cfg!(debug_assertions) && vtfx.has_onebit_alpha()
    {
        println!("[Debug] Has onebit alpha flag");
    }

    println!("{}", vtfx);

    let mut res_num = 0;
    let resource_entry_infos = vtfx.get_resource_entry_infos(&buffer);
    for resource in resource_entry_infos
    {
        println!("Reading resource #{}. Type: {:?}, Start: {}", res_num, resource.chTypeBytes, resource.resData);

        //Is this resource a high res image?
        if resource.chTypeBytes == VTF_LEGACY_RSRC_IMAGE
        {
            println!("    Type is VTF_LEGACY_RSRC_IMAGE");

            if !ARGS.no_resource_export
            {
                match resource_to_image(&buffer, &resource, &vtfx, &res_num) {
                    Ok(image) => {
                        let filename = path.file_stem().unwrap().to_str().unwrap();
                        let new_image_name = format!("{filename}_resource_{res_num}.png");
                        let save_path = &match ARGS.output.is_some()
                        {
                            true => ARGS.output.as_ref().unwrap().clone().join(&new_image_name),
                            false => PathBuf::from(&new_image_name)
                        };
                        image.save_with_format(save_path, image::ImageFormat::Png)?;
                        println!("    ✅ Saved resource image data to '{}'", save_path.as_path().to_string_lossy());

                        if ARGS.open
                        {
                            println!("    Opening image...");
                            opener::open(save_path.as_path())?;
                        }
                    },
                    Err(error) => {println!("    ❌ Error converting resource {} to image: {}", res_num, error)},
                }
            }
            
            res_num += 1;
        }
        else
        {
            println!("Error: Unknown resource type, skipping...");
        }
    }

    Ok(vtfx)
}

///Extract image resource and return it as DynamicImage
fn resource_to_image(buffer: &[u8], resource_entry_info: &ResourceEntryInfo, vtfx: &VTFXHEADER, res_num: &i32) -> Result<DynamicImage, Box<dyn Error>>
{
    let res_start: usize = resource_entry_info.resData.try_into()?;
    //Copy input buffer
    let mut resource_buffer = Vec::from(&buffer[res_start..buffer.len()]);
    let image_format = &vtfx.image_format;
    let format_info = image_format.get_format_info();
    if format_info.is_some()
    {
        let format_info_u = format_info.unwrap();

        if *image_format == ImageFormat::IMAGE_FORMAT_BGRA8888
        {
            println!("❕ NOTE: In this version ({}), BGRA8888 has its channel order changed to G,B,A,R. If this causes invalid results then please open a github issue.", VERSION)
        }

        let mut image_vec: Vec<u8>;

        let width = vtfx.width as usize;
        let height = vtfx.height as usize;

        let mut bc_read_offset = 0;

        println!("Resource #{res_num}: w: {width}, h: {height}");

        let is_lzma_compressed = &resource_buffer[0..4] == LZMA_MAGIC;

        //Is this resource lzma compressed?
        let expected_size = format_info_u.get_expected_size(&width, &height);
        if is_lzma_compressed
        {
            println!("    Image resource is LZMA compressed, decompressing...");
            //Decompress and replace resource buffer
            resource_buffer = decompress_lzma(&mut resource_buffer, expected_size, vtfx)?;
        }
        else if resource_buffer.len() < expected_size
        {
            let size_error = io::Error::new(io::ErrorKind::InvalidInput, format!("resource size is {} but expected length is {}, resource cannot decoded", resource_buffer.len(), expected_size));
            return Err(Box::new(size_error));
        }

        //If this format is BC encoded
        if format_info_u.bc_format.is_some()
        {
            let bc_format = format_info_u.bc_format.unwrap();
            //Allocate space for 4 channels
            image_vec = vec![0; width * height * 4];
            let image_vec_slice = image_vec.as_mut_slice();

            if vtfx.is_xbox() || ARGS.force_dxt_endian_fix
            {
                println!("    Applying endianness fix to resource '{res_num}' before dxt decode...");
                correct_dxt_endianness(&bc_format, &mut resource_buffer)?;
            }
            else
            {
                println!("! Will skip applying dxt endian fix for image resource '{res_num}' !")
            }

            if vtfx.mip_count > 1
            {
                println!("    Resource {res_num} contains {} mip levels, only mip 0 will be exported", vtfx.mip_count);
                bc_read_offset = resource_buffer.len() - vtfx.get_dxt_size()?;
            }

            println!("    Decoding image from {:?}, DTX buffer offset: {}", format_info_u.bc_format.unwrap(), bc_read_offset);
            //Decompress dxt image, if its still compressed this will fail
            //Use read offset when getting dtx buffer slice
            bc_format.decompress(&mut resource_buffer[bc_read_offset..], width, height, image_vec_slice);
        }
        else
        {
            if vtfx.is_xbox() && vtfx.mip_count > 1
            {
                #[cfg(debug_assertions)]
                {
                    println!("    [DEBUG] Will adjust image vector to start at largest mip map");
                }

                //Make new vector but skip to start of largest mip map (as img resources are packed smallest to largest dimension wise)
                let large_mip_start = resource_buffer.len() - expected_size;
                image_vec = resource_buffer[large_mip_start..].to_vec();
            }
            else
            {
                //Resource buffer is already usable
                image_vec = resource_buffer;
            }
        }

        //Take decompressed data and put into image
        let mut output_image = DynamicImage::new_rgba8(vtfx.width.into(), vtfx.height.into());

        if ARGS.export_alpha
        {
            println!("    Alpha will be included in the export for image resource '{res_num}'");
        }

        let width_u32 = width as u32;
        let depth_u32 = format_info_u.depth as u32;
        let channels = format_info_u.channels as usize;
        for y in 0..output_image.height()
        {
            for x in 0..output_image.width()
            {
                let mut pixel: Rgba<u8> = Rgba([255;4]);
                //Index of pixel data to read from decoded output
                let pixel_index = (x + y * width_u32) * depth_u32 * 4;
                for channel in 0..channels
                {
                    //Using format data, construct index and copy source image pixel colour data
                    let channel_offset = format_info_u.channel_order[channel] as u32;
                    //Add channel offset to pixel index.
                    let from_index: usize = (pixel_index + (channel_offset * depth_u32)) as usize;

                    if from_index < image_vec.len()
                    {
                        pixel[channel] = get_pixel_as_u8(&image_vec, from_index, &format_info_u.depth)?;
                    }
                }
                
                //Override alpha if not explicitly enabled
                if !ARGS.export_alpha
                {
                    pixel[3] = 255;
                }
                else if ARGS.experimental_onebitalpha
                {
                    pixel[3] = match pixel[3] > 1{
                        true => 0,
                        false => 255
                    };
                }
                
                output_image.put_pixel(x, y, pixel);
            }
        }

        Ok(output_image)
    }
    else
    {
        let err = io::Error::new(io::ErrorKind::Other, format!("Unsupported image format: {:?}.\nRequest for other formats to be supported on github.", image_format));
        return Err(Box::new(err));
    }
}

///Decompress resource that is compressed via lzma. Creates new buffer with new header.
fn decompress_lzma(resource_buffer: &mut Vec<u8>, expected_compressed_size: usize, vtfx: &VTFXHEADER) -> Result<Vec<u8>, Box<dyn Error>>
{
    //Read data from valves lzma header
    let actual_size: u64 = u32::from_le_bytes(resource_buffer[4..8].try_into()?).into();
    let mut decomp: Vec<u8> = Vec::with_capacity(actual_size as usize);
    let compressed_size: u32 = u32::from_le_bytes(resource_buffer[8..12].try_into()?);
    let mut dictionary_size: u32 = 0;
    let props_data = &resource_buffer[12..16];
    for i in 0..3
    {
        dictionary_size += (props_data[1 + i] as u32) << (i * 8);
    }
    if dictionary_size == 0
    {
        dictionary_size = 1;
    }

    //Reconstruct new header + data to decompress
    let mut new_header_resource_buffer: Vec<u8> = Vec::with_capacity(resource_buffer.len());
    new_header_resource_buffer.push(resource_buffer[12]);

    //Print valves properties if debug build
    #[cfg(debug_assertions)]
    {
        println!("[Debug LZMA] Dictionary size: {}", dictionary_size);
        let mut prop0: u8 = u8::from(resource_buffer[12]);
        let original_prop0 = prop0.clone();
        let mut pb: i32 = 0;
        let mut lp: i32 = 0;
        let mut lc: i32 = 0;
        get_valve_lzma_properties(&mut prop0, &mut pb, &mut lp, &mut lc);
        println!("[Debug LZMA] Properties Decoded ({}): prop0: {}, pb: {}, lp: {}, lc: {}", original_prop0, prop0, pb, lp, lc);
    }

    new_header_resource_buffer.extend_from_slice(&dictionary_size.to_le_bytes());
    new_header_resource_buffer.extend_from_slice(&actual_size.to_le_bytes());
    new_header_resource_buffer.extend_from_slice(&resource_buffer[17..]);
    *resource_buffer = new_header_resource_buffer;

    if cfg!(debug_assertions) {
        println!("[Debug LZMA] Actual size: {} (Expected {}). Compressed size: {} (header cmpr size {})", actual_size, expected_compressed_size, compressed_size, vtfx.compressed_size);
    }

    lzma_rs::lzma_decompress(&mut &resource_buffer[..], &mut decomp)?;
    println!("    Decompressed to: {}, Expected: {}", decomp.len(), expected_compressed_size);
    Ok(decomp)
}

///Get pixel as u8. Convert larger sized pixels down
fn get_pixel_as_u8(in_buffer: &Vec<u8>, index: usize, depth: &u16) -> Result<u8, Box<dyn Error>>
{
    match depth
    {
        1 => Ok(in_buffer[index]),
        2 => {
            let colour = u16::from_be_bytes(in_buffer[index..index+2].try_into()?);
            Ok((colour / 2).try_into()?)
        },
        4 => {
            let colour = u32::from_be_bytes(in_buffer[index..index+4].try_into()?);
            Ok((colour / 4).try_into()?)
        },
        _ => Err(Box::new(io::Error::new(io::ErrorKind::Other, format!("Unexpected depth size '{}'", depth))))
    }
}

/// Get lzma properties same way as source 2013
fn get_valve_lzma_properties(prop0: &mut u8, pb: &mut i32, lp: &mut i32, lc: &mut i32) 
{
    while pb < &mut 5 && prop0 >= & mut(9 * 5){
        *pb += 1;
        *prop0 -= 45;
    }
    
    // Second loop:
    while lp < &mut 5 && prop0 >= &mut 9 {
        *lp += 1;
        *prop0 -= 9;
    }

    *lc = *prop0 as i32;
}
