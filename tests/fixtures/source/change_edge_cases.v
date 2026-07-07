`timescale 1ns/1ns

module top;
  reg clk_r = 1'b0;
  reg clk1_r = 1'b0;
  reg clk2_r = 1'b0;
  wire clk = clk_r;
  wire clk1 = clk1_r;
  wire clk2 = clk2_r;

  initial begin
    $dumpfile("change_edge_cases.vcd");
    $dumpvars(0, clk, clk1, clk2);
    #5 clk_r = 1'b1;
    #5 begin
      clk_r = 1'b0;
      clk1_r = 1'b1;
    end
    #5 begin
      clk_r = 1'b1;
      clk2_r = 1'b1;
    end
    #5 begin
      clk_r = 1'bx;
      clk1_r = 1'b0;
    end
    #5 begin
      clk_r = 1'b0;
      clk2_r = 1'b0;
    end
    #5 begin
      clk_r = 1'b1;
      clk1_r = 1'b1;
      clk2_r = 1'b1;
    end
    #0 $finish;
  end
endmodule
