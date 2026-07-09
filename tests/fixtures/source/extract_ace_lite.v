`timescale 1ns/1ns

module top;
  reg clk = 1'b0;

  reg ace_lite_aw_valid_o = 1'b0;
  reg ace_lite_aw_ready_i = 1'b1;
  reg [1:0] ace_lite_aw_domain_o = 2'b00;
  reg [2:0] ace_lite_aw_snoop_o = 3'b000;
  reg [1:0] ace_lite_aw_bar_o = 2'b00;
  reg ace_lite_aw_unique_o = 1'b0;

  reg ace_lite_ar_valid_o = 1'b0;
  reg ace_lite_ar_ready_i = 1'b1;
  reg [1:0] ace_lite_ar_domain_o = 2'b00;
  reg [3:0] ace_lite_ar_snoop_o = 4'b0000;
  reg [1:0] ace_lite_ar_bar_o = 2'b00;

  initial begin
    $dumpfile("extract_ace_lite.vcd");
    $dumpvars(0, top);
    #4 begin
      ace_lite_aw_valid_o = 1'b1;
      ace_lite_aw_domain_o = 2'b10;
      ace_lite_aw_snoop_o = 3'b000;
      ace_lite_aw_bar_o = 2'b00;
      ace_lite_aw_unique_o = 1'b1;
      ace_lite_ar_valid_o = 1'b1;
      ace_lite_ar_domain_o = 2'b01;
      ace_lite_ar_snoop_o = 4'b0000;
      ace_lite_ar_bar_o = 2'b01;
    end
    #1 clk = 1'b1;
    #0 $finish;
  end
endmodule
