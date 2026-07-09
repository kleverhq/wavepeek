`timescale 1ns/1ns

module cpu(output wire valid);
  reg valid_r = 1'b0;
  assign valid = valid_r;

  initial begin
    #5 valid_r = 1'b1;
  end
endmodule

module mem(output wire ready);
  reg ready_r = 1'b0;
  assign ready = ready_r;

  initial begin
    #10 ready_r = 1'b1;
  end
endmodule

module top;
  reg clk_r = 1'b0;
  wire clk = clk_r;
  reg [7:0] data = 8'b00000000;
  parameter [7:0] cfg = 8'b10101010;

  cpu cpu();
  mem mem();

  initial begin
    $dumpfile("m2_core.vcd");
    $dumpvars(0, clk, data, cfg, cpu.valid, mem.ready);
    #5 clk_r = 1'b1;
    #5 data = 8'b00001111;
    #0 $finish;
  end
endmodule
