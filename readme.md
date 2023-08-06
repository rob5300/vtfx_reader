# VTFX Reader + Image Export
A tool to read the header + output image resources from a VTFX file. Written in rust.

Works with xbox 360 vtf files [*.360.vtf]. For PS3 files [*.vtf] you probably need to use the ``--no-dxt-fix`` argument.

Working texture export formats (Open issue to request):
- DXT1
- DXT5
- RGBA16161616

Compressed (LZMA) and non compressed images are supported. By default alpha is not exported, but can be enabled with the ``--export-alpha`` argument.

## Arguments
    -i, --input <INPUT>
            Input .vtf file path

    -o, --output <OUTPUT>
            Output folder

        --export-alpha
            Export alpha channel

        --mip0-only
            Try to output only mip 0 (EXPERIMENTAL)

        --no-dxt-fix
            Do not use big to little endian fix on DXT images

        --no-resource-export
            Do not export any resources

        --open
            Auto open exported images