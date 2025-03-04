module main(
    input clk,
    input rst,
    input en,
    output reg [31:0] count
);
  always @(posedge clk) begin
    if (rst) begin
      count <= 0;
    end else if (en) begin
      count <= count + 1;
    end
  end
endmodule
