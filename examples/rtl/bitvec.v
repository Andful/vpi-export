module main;
  integer i = 0;
  reg [31:0] a  = {32'b0};
  initial begin
    $bitvec(a, i);
    $display("%b", a);
    $display(i);
  end
endmodule