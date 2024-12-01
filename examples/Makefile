DEPENDENCY := ../target/release/deps
VPI_EXPORT := ../target/release/libvpi_export.rlib

%:	
	rustc  -O -C strip=symbols -C panic=abort --crate-type cdylib -L dependency=$(DEPENDENCY) --extern vpi_export=$(VPI_EXPORT) -o out/$@.vpi src/$@.rs
	iverilog -o out/$@.vvp rtl/$@.v
	vvp -Mout -m $@ out/$@.vvp