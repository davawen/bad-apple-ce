# Bad Apple CE

This is a program designed to play bad apple on a Ti 83/83+/84/84+ calculator.
It is composed of two parts:

## The Compressor

Written in rust, it takes the bad apple video split into images (assumed to be 160x120, 10 FPS) and compresses it heavily to fit on the calculator's flash memory.

The algorithm is basically fancy RLE (Run length encoding), with either 1 or 2 bytes dedicated to each strip of pixels:  
```
1 byte case:
*-------------------------------------------------------------*
| Marking bit (Off) | Count (6 bits) | Value (Black or white) |
*-------------------------------------------------------------*
=> With 63 or less consequent pixels

2 byte case:
*---------------------------*-------------------*
| Marking bit (On)  | Count | (14 bits) | Value |
*---------------------------*-------------------*
=> With between 64 and 16383 consequent pixels
```

The hybrid approach allows for lots of compression in areas with low frequency detail (only 4 bytes needed for an empty frame), while reducing overhead in areas with high frequency detail.  
Additionally, pixels are encoded vertically instead of horizontally to optimize for Bad Apple's characters.

Every compressed image is then concateneted together and outputted to a binary (in blocks of 65 KiB, the maximum size for a calculator file)

Every binary file generated just needs to be converted to an AppVar, and put on the calculator(through TiLP or TI Connect).
```bash
for f in ./*.bin
do
    convbin -j bin -k 8xv -i $f -o "f$(basename $f | cut -d. -f1)".8xv -n "f$(basename $f | cut -d. -f1)" -r;
done
````

## The Player

Written in C, it reads the AppVars from the calculator's memory, uncompresses them and draw them to the screen.  
It uses the calculator's clock to wait between frames.
