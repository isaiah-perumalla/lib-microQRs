# Simple QR encoder


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

