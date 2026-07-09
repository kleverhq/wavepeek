`timescale 1ns/1ns

module top;
  reg clk_r = 1'b0;
  reg aresetn_r = 1'b1;
  reg axi_aw_valid_o_r = 1'b0;
  reg axi_aw_ready_i_r = 1'b1;
  reg [7:0] axi_aw_addr_o_r = 8'h00;
  reg [2:0] axi_aw_prot_o_r = 3'h0;
  reg axi_w_valid_o_r = 1'b0;
  reg axi_w_ready_i_r = 1'b1;
  reg [7:0] axi_w_data_o_r = 8'h00;
  reg axi_w_strb_o_r = 1'b1;
  reg axi_b_valid_i_r = 1'b0;
  reg axi_b_ready_o_r = 1'b1;
  reg [1:0] axi_b_resp_i_r = 2'h0;
  reg axi_ar_valid_o_r = 1'b0;
  reg axi_ar_ready_i_r = 1'b1;
  reg [7:0] axi_ar_addr_o_r = 8'h00;
  reg [2:0] axi_ar_prot_o_r = 3'h0;
  reg axi_r_valid_i_r = 1'b0;
  reg axi_r_ready_o_r = 1'b1;
  reg [7:0] axi_r_data_i_r = 8'h00;
  reg [1:0] axi_r_resp_i_r = 2'h0;
  reg axi_misc_o_r = 1'b1;

  wire clk = clk_r;
  wire aresetn = aresetn_r;
  wire axi_aw_valid_o = axi_aw_valid_o_r;
  wire axi_aw_ready_i = axi_aw_ready_i_r;
  wire [7:0] axi_aw_addr_o = axi_aw_addr_o_r;
  wire [2:0] axi_aw_prot_o = axi_aw_prot_o_r;
  wire axi_w_valid_o = axi_w_valid_o_r;
  wire axi_w_ready_i = axi_w_ready_i_r;
  wire [7:0] axi_w_data_o = axi_w_data_o_r;
  wire axi_w_strb_o = axi_w_strb_o_r;
  wire axi_b_valid_i = axi_b_valid_i_r;
  wire axi_b_ready_o = axi_b_ready_o_r;
  wire [1:0] axi_b_resp_i = axi_b_resp_i_r;
  wire axi_ar_valid_o = axi_ar_valid_o_r;
  wire axi_ar_ready_i = axi_ar_ready_i_r;
  wire [7:0] axi_ar_addr_o = axi_ar_addr_o_r;
  wire [2:0] axi_ar_prot_o = axi_ar_prot_o_r;
  wire axi_r_valid_i = axi_r_valid_i_r;
  wire axi_r_ready_o = axi_r_ready_o_r;
  wire [7:0] axi_r_data_i = axi_r_data_i_r;
  wire [1:0] axi_r_resp_i = axi_r_resp_i_r;
  wire axi_misc_o = axi_misc_o_r;

  initial begin
    $dumpfile("extract_axi_lite.vcd");
    $dumpvars(
      0,
      clk,
      aresetn,
      axi_aw_valid_o,
      axi_aw_ready_i,
      axi_aw_addr_o,
      axi_aw_prot_o,
      axi_w_valid_o,
      axi_w_ready_i,
      axi_w_data_o,
      axi_w_strb_o,
      axi_b_valid_i,
      axi_b_ready_o,
      axi_b_resp_i,
      axi_ar_valid_o,
      axi_ar_ready_i,
      axi_ar_addr_o,
      axi_ar_prot_o,
      axi_r_valid_i,
      axi_r_ready_o,
      axi_r_data_i,
      axi_r_resp_i,
      axi_misc_o
    );
    #4 begin
      axi_aw_valid_o_r = 1'b1;
      axi_aw_addr_o_r = 8'h12;
      axi_aw_prot_o_r = 3'h2;
      axi_w_valid_o_r = 1'b1;
      axi_w_data_o_r = 8'haa;
      axi_b_valid_i_r = 1'b1;
      axi_b_resp_i_r = 2'h1;
      axi_ar_valid_o_r = 1'b1;
      axi_ar_addr_o_r = 8'h34;
      axi_ar_prot_o_r = 3'h3;
      axi_r_valid_i_r = 1'b1;
      axi_r_data_i_r = 8'h55;
      axi_r_resp_i_r = 2'h2;
    end
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;
    #3 begin
      aresetn_r = 1'b0;
      axi_aw_addr_o_r = 8'hff;
    end
    #1 clk_r = 1'b1;
    #0 $finish;
  end
endmodule
