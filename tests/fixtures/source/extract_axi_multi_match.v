`timescale 1ns/1ns

module top;
  wire clk = 1'b0;
  wire axi_awvalid_awready = 1'b0;

  initial begin
    $dumpfile("extract_axi_multi_match.vcd");
    $dumpvars(0, clk, axi_awvalid_awready);
    #0 $finish;
  end
endmodule
