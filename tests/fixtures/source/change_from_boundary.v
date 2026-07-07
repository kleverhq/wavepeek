`timescale 1ns/1ns

module top;
  reg sig_r = 1'b1;
  wire sig = sig_r;

  initial begin
    $dumpfile("change_from_boundary.vcd");
    $dumpvars(0, sig);
    #5 sig_r = 1'b0;
    #0 $finish;
  end
endmodule
