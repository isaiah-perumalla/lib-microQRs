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

## Benchmarks
`cargo +nightly bench`

## Profiling 
need to preserver frame pointer

`RUSTFLAGS='-C force-frame-pointers=y'`
Build the benchmark executable using command below
`RUSTFLAGS='-C force-frame-pointers=y' cargo +nightly bench --no-run`

### using Perf
`perf record -g `

`perf report -g graph,0.5,caller`

