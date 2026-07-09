`timescale 1ns/1ns

module top;
  integer i;
  reg sig_r = 1'b0;
  wire sig = sig_r;

  initial begin
    $dumpfile("change_many_events.vcd");
    $dumpvars(0, sig);
    for (i = 1; i <= 52; i = i + 1) begin
      #1 sig_r = ~sig_r;
    end
    #0 $finish;
  end
endmodule
