VPI_MODULE_PATH := ../target/debug/examples

%:	
	cargo build --example $@
	cp $(VPI_MODULE_PATH)/lib$@.so vpi/$@.vpi
	iverilog -o vvp/$@.vvp rtl/$@.v
	vvp -Mvpi -m $@ vvp/$@.vvp

