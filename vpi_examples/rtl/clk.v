module main(
  input clk
);
  initial begin
    #100000 $finish();
  end
  always @(posedge clk) begin
    $display("Hi from verilog");
  end
endmodule
