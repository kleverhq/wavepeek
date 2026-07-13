`timescale 1ns/1ns

module top;
  reg clk = 1'b0;

  reg ace5_lite_aw_valid_o = 1'b0;
  reg ace5_lite_aw_ready_i = 1'b1;
  reg [15:0] ace5_lite_aw_mecid_o = 16'h0000;
  reg ace5_lite_aw_mmu_valid_o = 1'b0;
  reg [1:0] ace5_lite_aw_mmu_flow_o = 2'h0;

  reg ace5_lite_w_valid_o = 1'b0;
  reg ace5_lite_w_ready_i = 1'b1;
  reg [7:0] ace5_lite_w_tag_update_o = 8'h00;

  reg ace5_lite_b_valid_i = 1'b0;
  reg ace5_lite_b_ready_o = 1'b1;
  reg [1:0] ace5_lite_b_tag_match_i = 2'h0;

  reg ace5_lite_ar_valid_o = 1'b0;
  reg ace5_lite_ar_ready_i = 1'b1;
  reg [15:0] ace5_lite_ar_mecid_o = 16'h0000;
  reg [1:0] ace5_lite_ar_tagop_o = 2'h0;

  reg ace5_lite_r_valid_i = 1'b0;
  reg ace5_lite_r_ready_o = 1'b1;
  reg [4:0] ace5_lite_r_chunknum_i = 5'h00;

  reg ace5_lite_aw_pending_o = 1'b0;
  reg ace5_lite_aw_valid_chk_o = 1'b0;
  reg ace5_lite_aw_actv_o = 1'b0;
  reg ace5_lite_ac_valid_i = 1'b0;
  reg ace5_lite_dvm_aw_valid_o = 1'b0;
  reg ace5_lite_dvm_aw_ready_i = 1'b1;
  reg ace5_lite_acp_aw_valid_o = 1'b0;
  reg ace5_lite_acp_aw_ready_i = 1'b1;

  initial begin
    $dumpfile("extract_ace5_lite.vcd");
    $dumpvars(0, top);
    #4 begin
      ace5_lite_aw_valid_o = 1'b1;
      ace5_lite_aw_mecid_o = 16'ha55a;
      ace5_lite_aw_mmu_valid_o = 1'b1;
      ace5_lite_aw_mmu_flow_o = 2'h2;
      ace5_lite_w_valid_o = 1'b1;
      ace5_lite_w_tag_update_o = 8'h3c;
      ace5_lite_b_valid_i = 1'b1;
      ace5_lite_b_tag_match_i = 2'h2;
      ace5_lite_ar_valid_o = 1'b1;
      ace5_lite_ar_mecid_o = 16'hb66b;
      ace5_lite_ar_tagop_o = 2'h1;
      ace5_lite_r_valid_i = 1'b1;
      ace5_lite_r_chunknum_i = 5'h07;
      ace5_lite_aw_pending_o = 1'b1;
      ace5_lite_aw_valid_chk_o = 1'b1;
      ace5_lite_aw_actv_o = 1'b1;
      ace5_lite_ac_valid_i = 1'b1;
      ace5_lite_dvm_aw_valid_o = 1'b1;
      ace5_lite_acp_aw_valid_o = 1'b1;
    end
    #1 clk = 1'b1;
    #0 $finish;
  end
endmodule
