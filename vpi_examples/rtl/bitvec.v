module main(
  input clk
);
  reg [29:0] a  = {29'b0};
  reg [1:0] b  = {2'b0};
  reg [31:0] conc  = {32'b0};
  initial begin
    $bitvec(a, b, conc);
    #10 $display("%b %b %b", a, b, conc);
    #10 b = 2'b11;
    $display("%b %b %b", a, b, conc);
    #10 b = 2'b00;
    $display("%b %b %b", a, b, conc);
  end

  always @(posedge clk ) begin
    $display("Hi from verilog");
  end
endmodule
