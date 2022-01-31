# dream86 - an x86/PC :computer: emulator

![alley cat screenshot](https://github.com/friol/dream86/raw/master/alleycat.png)

dream86, the x86/PC emulator dedicated to my father.<br/><br/>

When I was young, he bought a 386sx with MS-DOS 3.30.<br/>
dream86 is capable of running MS-DOS 3.30; it was the 1st OS that it booted. It took many nights of painstaking debugging, but, at the end, it's here.<br/>
<br/>
dream86 is written in Rust. It's an experiment. I started using this language in December 2021, when I was searching for a different language to do Advent Of Code (https://adventofcode.com/). I'm not proficient in this language, but at least now I know that you have to run "cargo run --release" if you want an executable that is not a snail.

To compile and run dream86:

```
cargo run --release <disk image full path> <com name> <runmode>
```

where: <br/>
<br/>
"disk image full path" is the path of a 1.44 .img disk image<br/>
"com name" is the name of a .com or .bin program (used only with runmode=1 or 2)<br/>
"runmode" is 0 to run the disk image, 1 to run the com file at the 2nd parameter and 2 to run a .bin file from artlav's test suite<br/>

Have fun with dream86!
