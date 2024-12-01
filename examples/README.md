# Run Example

```bash
cargo build
iverilog -o test.vvp test.v
cp ../target/debug/libvpi_example.so test.vpi
vvp -M. -m test test.vvp
```
