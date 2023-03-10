![Build](https://github.com/isaiah-perumalla/qrs/actions/workflows/rust.yml/badge.svg)
[![codecov](https://codecov.io/gh/isaiah-perumalla/lib-microQRs/branch/main/graph/badge.svg?token=H1K7VAIPXT)](https://codecov.io/gh/isaiah-perumalla/lib-microQRs)

## Example 
```shell
cargo run --example txt2qr "lib-µQRs is tiny efficient Rust library to encode to QR code" > qr.ppm

```
generates the image in ppm format 
![qr-img](./assets/qrs.png)

## Usage
```rust
let result = microQRs::encode::<144>("lib-µQRs is tiny efficient Rust library to encode to QR code");
        if let Ok(code) = result {
            microQRs::img::ppm::to_img(&code, [WHITE, BLACK], &mut stdout());
        } else {
            eprintln!("encode err ");
        }
```
## Motivation
goal of this project is to build something I find interesting using Rust
Recently been curious how QR codes work, in particular the Error Correction using finite fields seemed interesting for me ,
Rust is touted as being **blazingly fast and memory-efficient**, as with any programming language, I decided to build something I find interesting, and in the 
process I hope to learn tools the language provides control like memory management strategies and fine turne performance where needed. 
I have the following goals 
1. No 3rd Party libs everything is written from scratch using **only** the stdlib
2. minimize heap usage, take advantage of low-level control offered by rust and stack alloc where possible
3. learn about Galios Finite Fields, implement everything from ground up
4. learn how to benchmark Rust code and analyse performance


## Limitations
1. current only support for bytes
2. QR Level-1 to 5 support (need to implement block interleaving for it work on higher levels)

## Benchmarks
currently the standard benchmark lib is only available on nightly builds of Rust so need to run the following to execute benchmarks
if you dont have Rust nightly build use the following to install
`rustup toolchain install nightly`
To run the benchmarks run the following
`cargo +nightly bench --bench benchmark`

To run a specific bench, pass the args as below. it will run only benchmark begining with "bench_code"
`cargo +nightly bench -- bench_code`

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

`perf record -g -e L1-dcache-loads,L1-dcache-load-misses ./target/release/deps/benchmarks-12aee1ab12314b91 --bench`
`perf record -g `

`perf report -g "graph,0.5,caller"` 

can also try `perf report -g "fractal,0.5,caller"` 

## Flame-graphs for visual perspective

`cargo install --force inferno`

## ToDo
1. rust inferno flame graphs
2. perf cache misses etc
3. Zprint-type-size explore type sizes
