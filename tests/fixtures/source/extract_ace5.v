`timescale 1ns/1ns

module top;
  reg clk = 1'b0;

  reg ace5_aw_valid_o = 1'b0;
  reg ace5_aw_ready_i = 1'b1;
  reg ace5_aw_trace_o = 1'b0;

  reg ace5_w_valid_o = 1'b0;
  reg ace5_w_ready_i = 1'b1;
  reg ace5_w_poison_o = 1'b0;

  reg ace5_b_valid_i = 1'b0;
  reg ace5_b_ready_o = 1'b1;
  reg ace5_b_idunq_i = 1'b0;

  reg ace5_ar_valid_o = 1'b0;
  reg ace5_ar_ready_i = 1'b1;
  reg [3:0] ace5_ar_vmidext_o = 4'h0;

  reg ace5_r_valid_i = 1'b0;
  reg ace5_r_ready_o = 1'b1;
  reg ace5_r_poison_i = 1'b0;

  reg ace5_ac_valid_i = 1'b0;
  reg ace5_ac_ready_o = 1'b1;
  reg [3:0] ace5_ac_vmidext_i = 4'h0;

  reg ace5_cr_valid_o = 1'b0;
  reg ace5_cr_ready_i = 1'b1;
  reg [3:0] ace5_cr_nsaid_o = 4'h0;

  reg ace5_cd_valid_o = 1'b0;
  reg ace5_cd_ready_i = 1'b1;
  reg ace5_cd_poison_o = 1'b0;

  initial begin
    $dumpfile("extract_ace5.vcd");
    $dumpvars(0, top);
    #4 begin
      ace5_aw_valid_o = 1'b1;
      ace5_aw_trace_o = 1'b1;
      ace5_w_valid_o = 1'b1;
      ace5_w_poison_o = 1'b1;
      ace5_b_valid_i = 1'b1;
      ace5_b_idunq_i = 1'b1;
      ace5_ar_valid_o = 1'b1;
      ace5_ar_vmidext_o = 4'hd;
      ace5_r_valid_i = 1'b1;
      ace5_r_poison_i = 1'b1;
      ace5_ac_valid_i = 1'b1;
      ace5_ac_vmidext_i = 4'ha;
      ace5_cr_valid_o = 1'b1;
      ace5_cr_nsaid_o = 4'h7;
      ace5_cd_valid_o = 1'b1;
      ace5_cd_poison_o = 1'b1;
    end
    #1 clk = 1'b1;
    #0 $finish;
  end
endmodule
