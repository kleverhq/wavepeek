`timescale 1ns/1ns

module core(output wire execute);
  reg execute_r = 1'b0;
  assign execute = execute_r;

  initial begin
    #5 execute_r = 1'b1;
  end
endmodule

module cpu(output wire valid);
  reg valid_r = 1'b0;
  assign valid = valid_r;

  core core();

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
  wire reset_n = 1'b1;

  cpu cpu();
  mem mem();

  initial begin
    $dumpfile("signal_recursive_depth.vcd");
    $dumpvars(0, clk, reset_n, cpu.valid, cpu.core.execute, mem.ready);
    #5 clk_r = 1'b1;
    #5;
    #0 $finish;
  end
endmodule
