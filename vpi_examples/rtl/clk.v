module main(
  input clk
);
  always @(posedge clk) begin
    $display("Hi from verilog");
  end
endmodule
