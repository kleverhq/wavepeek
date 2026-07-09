`timescale 1ns/1ns

module top;
  reg clk_r = 1'b0;
  reg axi_wvalid_r = 1'b0;
  reg axi_wready_r = 1'b1;
  reg [3:0] axi_wid_r = 4'h0;
  reg [7:0] axi_wdata_r = 8'h00;

  wire clk = clk_r;
  wire axi_wvalid = axi_wvalid_r;
  wire axi_wready = axi_wready_r;
  wire [3:0] axi_wid = axi_wid_r;
  wire [7:0] axi_wdata = axi_wdata_r;

  initial begin
    $dumpfile("extract_axi3_w.vcd");
    $dumpvars(0, clk, axi_wvalid, axi_wready, axi_wid, axi_wdata);
    #4 begin
      axi_wvalid_r = 1'b1;
      axi_wid_r = 4'ha;
      axi_wdata_r = 8'hcc;
    end
    #1 clk_r = 1'b1;
    #0 $finish;
  end
endmodule
