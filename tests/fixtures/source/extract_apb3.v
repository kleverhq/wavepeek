`timescale 1ns/1ns

module top;
  reg clk_r = 1'b0;
  reg psel_r = 1'b0;
  reg penable_r = 1'b0;
  reg pwrite_r = 1'b1;
  reg pready_r = 1'b1;
  reg [7:0] paddr_r = 8'h10;
  reg [7:0] pwdata_r = 8'h3a;
  reg [7:0] prdata_r = 8'h00;
  reg pslverr_r = 1'b0;

  wire apb3_p_clk_i = clk_r;
  wire apb3_psel_o = psel_r;
  wire apb3_penable_o = penable_r;
  wire apb3_pwrite_o = pwrite_r;
  wire apb3_pready_i = pready_r;
  wire [7:0] apb3_paddr_o = paddr_r;
  wire [7:0] apb3_pwdata_o = pwdata_r;
  wire [7:0] apb3_prdata_i = prdata_r;
  wire apb3_pslverr_i = pslverr_r;

  initial begin
    $dumpfile("extract_apb3.vcd");
    $dumpvars(
      0,
      apb3_p_clk_i,
      apb3_psel_o,
      apb3_penable_o,
      apb3_pwrite_o,
      apb3_pready_i,
      apb3_paddr_o,
      apb3_pwdata_o,
      apb3_prdata_i,
      apb3_pslverr_i
    );
    #4 psel_r = 1'b1;
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;
    #3 penable_r = 1'b1;
    #1 clk_r = 1'b1;
    #0 $finish;
  end
endmodule
