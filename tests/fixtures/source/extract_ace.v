`timescale 1ns/1ns

module top;
  reg clk = 1'b0;

  reg ace_aw_valid_o = 1'b0;
  reg ace_aw_ready_i = 1'b1;
  reg [1:0] ace_aw_domain_o = 2'b00;
  reg ace_aw_unique_o = 1'b0;

  reg ace_w_valid_o = 1'b0;
  reg ace_w_ready_i = 1'b1;
  reg [7:0] ace_w_data_o = 8'h00;

  reg ace_b_valid_i = 1'b0;
  reg ace_b_ready_o = 1'b1;
  reg [1:0] ace_b_resp_i = 2'b00;

  reg ace_ar_valid_o = 1'b0;
  reg ace_ar_ready_i = 1'b1;
  reg [1:0] ace_ar_domain_o = 2'b00;
  reg [1:0] ace_ar_bar_o = 2'b00;

  reg ace_r_valid_i = 1'b0;
  reg ace_r_ready_o = 1'b1;
  reg [3:0] ace_r_resp_i = 4'b0000;

  reg ace_ac_valid_i = 1'b0;
  reg ace_ac_ready_o = 1'b1;
  reg [15:0] ace_ac_addr_i = 16'h0000;

  reg ace_cr_valid_o = 1'b0;
  reg ace_cr_ready_i = 1'b1;
  reg [4:0] ace_cr_resp_o = 5'b00000;

  reg ace_cd_valid_o = 1'b0;
  reg ace_cd_ready_i = 1'b1;
  reg [7:0] ace_cd_data_o = 8'h00;
  reg ace_cd_last_o = 1'b0;

  initial begin
    $dumpfile("extract_ace.vcd");
    $dumpvars(0, top);
    #4 begin
      ace_aw_valid_o = 1'b1;
      ace_aw_domain_o = 2'b10;
      ace_aw_unique_o = 1'b1;
      ace_w_valid_o = 1'b1;
      ace_w_data_o = 8'ha1;
      ace_b_valid_i = 1'b1;
      ace_b_resp_i = 2'b01;
      ace_ar_valid_o = 1'b1;
      ace_ar_domain_o = 2'b11;
      ace_ar_bar_o = 2'b01;
      ace_r_valid_i = 1'b1;
      ace_r_resp_i = 4'b1101;
      ace_ac_valid_i = 1'b1;
      ace_ac_addr_i = 16'h1234;
      ace_cr_valid_o = 1'b1;
      ace_cr_resp_o = 5'b10101;
      ace_cd_valid_o = 1'b1;
      ace_cd_data_o = 8'hc3;
      ace_cd_last_o = 1'b1;
    end
    #1 clk = 1'b1;
    #0 $finish;
  end
endmodule
