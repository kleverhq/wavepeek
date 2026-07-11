`timescale 1ns/1ns

module top;
  wire clk = 1'b0;
  wire axi_a_awvalid_o = 1'b0;
  wire axi_b_awvalid_o = 1'b0;
  wire axi_a_arvalid_o = 1'b0;
  wire axi_b_arvalid_o = 1'b0;

  initial begin
    $dumpfile("extract_axi_multi_ambiguous.vcd");
    $dumpvars(0, clk, axi_a_awvalid_o, axi_b_awvalid_o, axi_a_arvalid_o, axi_b_arvalid_o);
    #0 $finish;
  end
endmodule
