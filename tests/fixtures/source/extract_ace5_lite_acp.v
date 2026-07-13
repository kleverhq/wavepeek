`timescale 1ns/1ns

module top;
  reg clk = 1'b0;

  reg ace5_lite_acp_aw_valid_o = 1'b0;
  reg ace5_lite_acp_aw_ready_i = 1'b1;
  reg [7:0] ace5_lite_acp_aw_len_o = 8'h00;
  reg [3:0] ace5_lite_acp_aw_snoop_o = 4'h0;
  reg [10:0] ace5_lite_acp_aw_stash_nid_o = 11'h000;
  reg [10:0] ace5_lite_acp_aw_mpam_o = 11'h000;

  reg ace5_lite_acp_w_valid_o = 1'b0;
  reg ace5_lite_acp_w_ready_i = 1'b1;
  reg ace5_lite_acp_w_last_o = 1'b0;

  reg ace5_lite_acp_b_valid_i = 1'b0;
  reg ace5_lite_acp_b_ready_o = 1'b1;
  reg ace5_lite_acp_b_id_unq_i = 1'b0;

  reg ace5_lite_acp_ar_valid_o = 1'b0;
  reg ace5_lite_acp_ar_ready_i = 1'b1;
  reg [7:0] ace5_lite_acp_ar_len_o = 8'h00;
  reg ace5_lite_acp_ar_chunken_o = 1'b0;
  reg [3:0] ace5_lite_acp_ar_snoop_o = 4'h0;

  reg ace5_lite_acp_r_valid_i = 1'b0;
  reg ace5_lite_acp_r_ready_o = 1'b1;
  reg ace5_lite_acp_r_last_i = 1'b0;
  reg [4:0] ace5_lite_acp_r_chunknum_i = 5'h00;
  reg [15:0] ace5_lite_acp_r_chunk_strb_i = 16'h0000;

  reg [2:0] ace5_lite_acp_aw_size_o = 3'h0;
  reg [1:0] ace5_lite_acp_aw_burst_o = 2'h0;
  reg ace5_lite_acp_w_tag_o = 1'b0;
  reg ace5_lite_acp_b_comp_i = 1'b0;
  reg [2:0] ace5_lite_acp_ar_size_o = 3'h0;
  reg ace5_lite_acp_r_tag_i = 1'b0;
  reg ace5_lite_acp_ac_valid_i = 1'b0;
  reg ace5_lite_acp_aw_pending_o = 1'b0;
  reg ace5_lite_acp_aw_valid_chk_o = 1'b0;

  initial begin
    $dumpfile("extract_ace5_lite_acp.vcd");
    $dumpvars(0, top);
    #4 begin
      ace5_lite_acp_aw_valid_o = 1'b1;
      ace5_lite_acp_aw_len_o = 8'h03;
      ace5_lite_acp_aw_snoop_o = 4'h1;
      ace5_lite_acp_aw_stash_nid_o = 11'h321;
      ace5_lite_acp_aw_mpam_o = 11'h456;
      ace5_lite_acp_w_valid_o = 1'b1;
      ace5_lite_acp_w_last_o = 1'b1;
      ace5_lite_acp_b_valid_i = 1'b1;
      ace5_lite_acp_b_id_unq_i = 1'b1;
      ace5_lite_acp_ar_valid_o = 1'b1;
      ace5_lite_acp_ar_len_o = 8'h03;
      ace5_lite_acp_ar_chunken_o = 1'b1;
      ace5_lite_acp_ar_snoop_o = 4'h2;
      ace5_lite_acp_r_valid_i = 1'b1;
      ace5_lite_acp_r_last_i = 1'b1;
      ace5_lite_acp_r_chunknum_i = 5'h0b;
      ace5_lite_acp_r_chunk_strb_i = 16'h5aa5;
      ace5_lite_acp_aw_size_o = 3'h4;
      ace5_lite_acp_aw_burst_o = 2'h1;
      ace5_lite_acp_w_tag_o = 1'b1;
      ace5_lite_acp_b_comp_i = 1'b1;
      ace5_lite_acp_ar_size_o = 3'h4;
      ace5_lite_acp_r_tag_i = 1'b1;
      ace5_lite_acp_ac_valid_i = 1'b1;
      ace5_lite_acp_aw_pending_o = 1'b1;
      ace5_lite_acp_aw_valid_chk_o = 1'b1;
    end
    #1 clk = 1'b1;
    #0 $finish;
  end
endmodule
