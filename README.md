![Build](https://github.com/isaiah-perumalla/qrs/actions/workflows/rust.yml/badge.svg)
# QR encoder implemented in Rust (no dependecies)
goal of this project is to build something I find interesting using Rust
Recently been curious how QR codes work, in particular the Error Correction using finite fields seemed interesting for me ,
Rust is touted as being **blazingly fast and memory-efficient**, as with any programming language, I decided to build something I find interesting, and in the 
process I hope to learn tools the language provides control like memory management strategies and fine turne performance where needed. 
I have following goals 
1. No 3rd Party libs everything is written from scratch using **only** the stdlib
2. minimize heap usage, take advantage of low-level control offered by rust and stack alloc where possible
3. learn about galios finite fields, implement everything from ground up
4. learn how to benchmark Rust code and analyse performance


## Limitations
1. current only support for bytes
2. QR Level-1 to 5 support (need to implement block interleaving for it work on higher levels)

## Benchmarks
currently the standard benchmark lib is only available on nightly builds of Rust so need to run the following to execute benchmarks
if you dont have Rust nightly build use the following to install
`rustup toolchain install nightly`
To run the benchmarks run the following
`cargo +nightly bench`

## Profiling 
Profiler need frame pointer to workout the call graph, on x86 this is by conventions store in ebp register, which indicates the starting address of the function’s stack frame
in release build the Rust omits storing the frame pointer, this causes issue for profiles when working out the call stack
to avoid this we need to set the flag below to ensure rust compiler preserves the frame pointer

`RUSTFLAGS='-C force-frame-pointers=y'`
Build the benchmark executable using command below
`RUSTFLAGS='-C force-frame-pointers=y' cargo +nightly bench --no-run`

### using Perf
Run `perf stat target/release/deps/benchmarks-xxx --bench`
this should provide an overall picture of how the program performs 

`perf record -g `

`perf report -g "graph,0.5,caller"` 

can also try `perf report -g "fractal,0.5,caller"` 

