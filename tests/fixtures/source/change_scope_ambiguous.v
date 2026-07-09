`timescale 1ns/1ns

module nested_top(output wire clk);
  reg clk_r = 1'b1;
  assign clk = clk_r;

  initial begin
    #5 clk_r = 1'b0;
  end
endmodule

module top;
  reg clk_r = 1'b0;
  wire clk = clk_r;

  nested_top top();

  initial begin
    $dumpfile("change_scope_ambiguous.vcd");
    $dumpvars(0, clk, top.clk);
    #5 clk_r = 1'b1;
    #0 $finish;
  end
endmodule
