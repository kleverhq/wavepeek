`timescale 1ns/1ns

module top;
  string message;
  real temperature;
  reg clk;
  event tick;

  initial begin
    message = "go";
    temperature = 0.5;
    clk = 0;
    #5;
    message = "stop";
    temperature = 2.5;
    -> tick;
    clk = 1;
    #5;
    message = "go";
    clk = 0;
    #5 $finish;
  end
endmodule
