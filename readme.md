# VTFX Image Converter (Source engine PS3/Xbox 360 VTF)
[![Download Latest Windows EXE](https://img.shields.io/badge/Download_Latest-Windows_EXE-orange?style=flat)](https://github.com/rob5300/vtfx_reader/releases/latest/download/vtfx_reader.exe)

[![GitHub Downloads (all assets, latest release)](https://img.shields.io/github/downloads/rob5300/vtfx_reader/latest/total?sort=date)
](https://github.com/rob5300/vtfx_reader/releases/latest)

VTFX Reader converts [VTFX files](https://developer.valvesoftware.com/wiki/VTFX_file_format) to PNG's. Supports LZMA compressed resources as well as big endian formatted resources. Written in rust.

- Supports 360 vtf files [\*.360.vtf]. DXT endianness is automatically fixed.
- PS3 files  [\*.vtf].

> [!NOTE]
> Xbox 360 vtfx's usually have multiple mip levels packed into the main resource, the largest(best) mip level will be exported.

## Why does this exist?
Common tools such as VTFEdit only support vtf files made for PC versions of source engine. PS3 and X360 versions of source use [VTFX](https://developer.valvesoftware.com/wiki/VTFX_file_format) which are specially formatted for these platforms. This means they cannot be read by existing tools without special logic.

## Working texture export formats (Open issue to request):
- DXT1
- DXT5
- RGBA16161616
- BGRX8888
- RGBA8888
- ABGR8888
- RGB888
- BGR888
- ARGB8888
- BGRA8888

Untested support for:

- LINEAR_BGRX8888.

Compressed (LZMA) and non compressed images are supported. By default alpha is not exported, but can be enabled with the ``--export-alpha`` argument.

Files detected to be for the xbox 360 (v 864.8) that are in the image formats IMAGE_FORMAT_DXT1, IMAGE_FORMAT_DXT3 or IMAGE_FORMAT_DXT5 will have their endianness converted before decoding (otherwise the output will have corrupted color). PS3 files (usually v 819.8) do not need this.

## How to use
Download the latest release and run via cmd/powershell/terminal using the command line arguments listed below to specify the input files and options.

e.g. ``./vtfx_reader -i foo.vtf`` to process the file "foo.vtf" in the same folder

    Usage: vtfx_reader.exe [OPTIONS] --input <INPUT>

    Options:
    -i, --input <INPUT>
            Input path (process single file) or folder (processes all vtf files in folder)

    -o, --output <OUTPUT>
            Output folder for exported images

        --export-alpha
            If alpha should be included in image export

        --force-dxt-endian-fix
            Force apply big to little endian fix on DXT image resources (otherwise automatic)

        --no-resource-export
            Do not export any resources

        --open
            Auto open exported images

    -h, --help
            Print help (see a summary with '-h')

    -V, --version
            Print version

## Download
Download a windows build from the [latest release](https://github.com/rob5300/vtfx_reader/releases/latest).

Other platforms should compile with the instructions below (linux builds may be added in future)

## Compiling
To compile from source, install the rust tooling [rustup](https://rustup.rs/), clone this project repo then use ``cargo run`` to build and run the project.

[texpresso](https://crates.io/crates/texpresso) is used to decode dxt data, and [lzma-rs](https://crates.io/crates/lzma-rs) for lzma decompression.

## Technical Info
To parse [VTFX](https://developer.valvesoftware.com/wiki/VTFX_file_format) files correctly, we parse the header contents manually while performing big to little endian conversion (rust thankfully provides this for most major data types). Additionally, images using DXT compression also require data to be converted from big to little endian before being uncompressed. This is custom written for BC1 and BC3 data blocks.

Thankfully no endian conversion is required for LZMA compressed data but some data does need to be adjusted for decompression to work correctly with [lzma-rs](https://crates.io/crates/lzma-rs).