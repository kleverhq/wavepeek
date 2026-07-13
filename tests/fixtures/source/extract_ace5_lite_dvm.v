`timescale 1ns/1ns

module top;
  reg clk = 1'b0;

  reg ace5_lite_dvm_aw_valid_o = 1'b0;
  reg ace5_lite_dvm_aw_ready_i = 1'b1;
  reg [15:0] ace5_lite_dvm_aw_mecid_o = 16'h0000;
  reg [3:0] ace5_lite_dvm_aw_nsaid_o = 4'h0;

  reg ace5_lite_dvm_w_valid_o = 1'b0;
  reg ace5_lite_dvm_w_ready_i = 1'b1;
  reg [7:0] ace5_lite_dvm_w_tag_update_o = 8'h00;

  reg ace5_lite_dvm_b_valid_i = 1'b0;
  reg ace5_lite_dvm_b_ready_o = 1'b1;
  reg [1:0] ace5_lite_dvm_b_resp_i = 2'h0;

  reg ace5_lite_dvm_ar_valid_o = 1'b0;
  reg ace5_lite_dvm_ar_ready_i = 1'b1;
  reg [15:0] ace5_lite_dvm_ar_mecid_o = 16'h0000;
  reg [1:0] ace5_lite_dvm_ar_tagop_o = 2'h0;

  reg ace5_lite_dvm_r_valid_i = 1'b0;
  reg ace5_lite_dvm_r_ready_o = 1'b1;
  reg [4:0] ace5_lite_dvm_r_chunknum_i = 5'h00;

  reg ace5_lite_dvm_ac_valid_i = 1'b0;
  reg ace5_lite_dvm_ac_ready_o = 1'b1;
  reg [31:0] ace5_lite_dvm_ac_addr_i = 32'h00000000;
  reg [3:0] ace5_lite_dvm_ac_vmidext_i = 4'h0;
  reg ace5_lite_dvm_ac_trace_i = 1'b0;

  reg ace5_lite_dvm_cr_valid_o = 1'b0;
  reg ace5_lite_dvm_cr_ready_i = 1'b1;
  reg ace5_lite_dvm_cr_trace_o = 1'b0;

  reg ace5_lite_dvm_aw_mmu_valid_o = 1'b0;
  reg ace5_lite_dvm_ar_mmu_valid_o = 1'b0;
  reg ace5_lite_dvm_b_tag_match_i = 1'b0;
  reg [3:0] ace5_lite_dvm_ac_snoop_i = 4'h0;
  reg [2:0] ace5_lite_dvm_ac_prot_i = 3'h0;
  reg [4:0] ace5_lite_dvm_cr_resp_o = 5'h00;
  reg ace5_lite_dvm_cd_valid_o = 1'b0;
  reg ace5_lite_dvm_aw_pending_o = 1'b0;
  reg ace5_lite_dvm_ac_valid_chk_i = 1'b0;

  initial begin
    $dumpfile("extract_ace5_lite_dvm.vcd");
    $dumpvars(0, top);
    #4 begin
      ace5_lite_dvm_aw_valid_o = 1'b1;
      ace5_lite_dvm_aw_mecid_o = 16'hc77c;
      ace5_lite_dvm_aw_nsaid_o = 4'h6;
      ace5_lite_dvm_w_valid_o = 1'b1;
      ace5_lite_dvm_w_tag_update_o = 8'h5a;
      ace5_lite_dvm_b_valid_i = 1'b1;
      ace5_lite_dvm_b_resp_i = 2'h1;
      ace5_lite_dvm_ar_valid_o = 1'b1;
      ace5_lite_dvm_ar_mecid_o = 16'hd88d;
      ace5_lite_dvm_ar_tagop_o = 2'h3;
      ace5_lite_dvm_r_valid_i = 1'b1;
      ace5_lite_dvm_r_chunknum_i = 5'h09;
      ace5_lite_dvm_ac_valid_i = 1'b1;
      ace5_lite_dvm_ac_addr_i = 32'h12345678;
      ace5_lite_dvm_ac_vmidext_i = 4'ha;
      ace5_lite_dvm_ac_trace_i = 1'b1;
      ace5_lite_dvm_cr_valid_o = 1'b1;
      ace5_lite_dvm_cr_trace_o = 1'b1;
      ace5_lite_dvm_aw_mmu_valid_o = 1'b1;
      ace5_lite_dvm_ar_mmu_valid_o = 1'b1;
      ace5_lite_dvm_b_tag_match_i = 1'b1;
      ace5_lite_dvm_ac_snoop_i = 4'hf;
      ace5_lite_dvm_ac_prot_i = 3'h7;
      ace5_lite_dvm_cr_resp_o = 5'h1f;
      ace5_lite_dvm_cd_valid_o = 1'b1;
      ace5_lite_dvm_aw_pending_o = 1'b1;
      ace5_lite_dvm_ac_valid_chk_i = 1'b1;
    end
    #1 clk = 1'b1;
    #0 $finish;
  end
endmodule
