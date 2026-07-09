`timescale 1ns/1ns

module top;
  wire clk = 1'b0;
  wire axi_awvalid_o = 1'b0;
  wire other_awvalid_o = 1'b0;

  initial begin
    $dumpfile("extract_axi_ambiguous.vcd");
    $dumpvars(0, clk, axi_awvalid_o, other_awvalid_o);
    #0 $finish;
  end
endmodule
