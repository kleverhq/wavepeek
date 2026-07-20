`timescale 1ns/1ns

module top;
  reg clk_r = 1'b0;
  reg resetn_r = 1'b1;
  reg psel_r = 1'b0;
  reg penable_r = 1'b0;
  reg pwrite_r = 1'b1;
  reg pready_r = 1'b1;
  reg [15:0] paddr_r = 16'h5000;
  reg [2:0] pprot_r = 3'h3;
  reg pnse_r = 1'b1;
  reg [3:0] pauser_r = 4'ha;
  reg [15:0] pwdata_r = 16'hbeef;
  reg [1:0] pstrb_r = 2'h3;
  reg [3:0] pwuser_r = 4'h5;
  reg [15:0] prdata_r = 16'h1234;
  reg pslverr_r = 1'b0;
  reg [3:0] pruser_r = 4'h6;
  reg [2:0] pbuser_r = 3'h7;

  wire apb5_pclk_i = clk_r;
  wire apb5_presetn_i = resetn_r;
  wire apb5_psel_o = psel_r;
  wire apb5_penable_o = penable_r;
  wire apb5_pwrite_o = pwrite_r;
  wire apb5_pready_i = pready_r;
  wire [15:0] apb5_paddr_o = paddr_r;
  wire [2:0] apb5_pprot_o = pprot_r;
  wire apb5_pnse_o = pnse_r;
  wire [3:0] apb5_pauser_o = pauser_r;
  wire [15:0] apb5_pwdata_o = pwdata_r;
  wire [1:0] apb5_pstrb_o = pstrb_r;
  wire [3:0] apb5_pwuser_o = pwuser_r;
  wire [15:0] apb5_prdata_i = prdata_r;
  wire apb5_pslverr_i = pslverr_r;
  wire [3:0] apb5_pruser_i = pruser_r;
  wire [2:0] apb5_pbuser_i = pbuser_r;
  wire apb5_pwakeup_o = 1'b1;
  wire apb5_paddrchk_o = 2'b00;

  initial begin
    $dumpfile("extract_apb5.vcd");
    $dumpvars(
      0,
      apb5_pclk_i,
      apb5_presetn_i,
      apb5_psel_o,
      apb5_penable_o,
      apb5_pwrite_o,
      apb5_pready_i,
      apb5_paddr_o,
      apb5_pprot_o,
      apb5_pnse_o,
      apb5_pauser_o,
      apb5_pwdata_o,
      apb5_pstrb_o,
      apb5_pwuser_o,
      apb5_prdata_i,
      apb5_pslverr_i,
      apb5_pruser_i,
      apb5_pbuser_i,
      apb5_pwakeup_o,
      apb5_paddrchk_o
    );
    #4 psel_r = 1'b1;
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;
    #3 penable_r = 1'b1;
    #1 clk_r = 1'b1;
    #0 $finish;
  end
endmodule
