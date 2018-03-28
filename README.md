# blkview
A tool to visualize blktrace output written in Rust.

## Usage
`blkview` generates GIFs from blktrace output files. Here is an example GIF from a trace of file creation on F2FS (light gray = read, dark gray = writes):

![](https://github.com/souvik1997/blkview/blob/master/example.gif)

```
blkview 0.1.0

USAGE:
    blkview -c <chunksize> -o <output> [-- <files>...]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c <chunksize>        
    -o <output>           

ARGS:
    <files>...
```

