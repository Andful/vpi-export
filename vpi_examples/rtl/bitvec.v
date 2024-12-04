module main;
  integer i = 0;
  reg [29:0] a  = {29'b0};
  reg [1:0] b  = {2'b0};
  reg [31:0] conc  = {32'b0};
  initial begin
    $bitvec(a, b, conc);
    $display("%b %b %b", a, b, conc);
    $display(i);
  end
endmodule