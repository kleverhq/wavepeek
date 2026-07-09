`timescale 1ns/1ns

module top;
  wire clk = 1'b0;
  wire axi_awvalid_a = 1'b0;
  wire axi_awvalid_b = 1'b0;
  wire axi_arvalid_a = 1'b0;
  wire axi_arvalid_b = 1'b0;

  initial begin
    $dumpfile("extract_axi_multi_ambiguous.vcd");
    $dumpvars(0, clk, axi_awvalid_a, axi_awvalid_b, axi_arvalid_a, axi_arvalid_b);
    #0 $finish;
  end
endmodule
