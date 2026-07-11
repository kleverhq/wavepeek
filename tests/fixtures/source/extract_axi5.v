`timescale 1ns/1ns

module top;
  reg clk = 1'b0;

  reg axi5_aw_valid_o = 1'b0;
  reg axi5_aw_ready_i = 1'b0;
  reg axi5_aw_mmu_valid_o = 1'b0;
  reg [15:0] axi5_aw_mecid_o = 16'h0000;
  reg axi5_aw_actv_o = 1'b0;
  reg [2:0] axi5_aw_prot_o = 3'h0;
  reg axi5_aw_nse_o = 1'b0;

  reg axi5_w_valid_o = 1'b0;
  reg axi5_w_ready_i = 1'b0;
  reg [3:0] axi5_w_tag_update_o = 4'h0;

  reg axi5_b_valid_i = 1'b0;
  reg axi5_b_ready_o = 1'b0;
  reg [1:0] axi5_b_tag_match_i = 2'h0;

  reg axi5_ar_valid_o = 1'b0;
  reg axi5_ar_ready_i = 1'b0;
  reg [15:0] axi5_ar_mecid_o = 16'h0000;
  reg axi5_ar_chunken_o = 1'b0;

  reg axi5_r_valid_i = 1'b0;
  reg axi5_r_ready_o = 1'b0;
  reg [4:0] axi5_r_chunknum_i = 5'h00;

  reg axi5_ac_valid_i = 1'b0;
  reg axi5_ac_ready_o = 1'b0;
  reg [31:0] axi5_ac_addr_i = 32'h00000000;
  reg [3:0] axi5_ac_vmidext_i = 4'h0;

  reg axi5_cr_valid_o = 1'b0;
  reg axi5_cr_ready_i = 1'b0;
  reg axi5_cr_trace_o = 1'b0;

  reg axi5_aw_pending_o = 1'b0;
  reg axi5_aw_valid_chk_o = 1'b0;
  reg axi5_cd_valid_o = 1'b0;
  reg axi5_awakeup_o = 1'b0;
  reg axi5_varqosaccept_i = 1'b0;
  reg axi5_syscoreq_o = 1'b0;
  reg axi5_broadcastatomic_i = 1'b0;
  reg axi5_activatereq_o = 1'b0;

  initial begin
    $dumpfile("extract_axi5.vcd");
    $dumpvars(0, top);

    #4 begin
      axi5_aw_valid_o = 1'b1;
      axi5_aw_ready_i = 1'b1;
      axi5_aw_mmu_valid_o = 1'b1;
      axi5_aw_mecid_o = 16'ha55a;
      axi5_aw_actv_o = 1'b1;
      axi5_aw_prot_o = 3'h5;
      axi5_aw_nse_o = 1'b1;
      axi5_w_valid_o = 1'b1;
      axi5_w_tag_update_o = 4'hc;
      axi5_b_ready_o = 1'b1;
      axi5_r_ready_o = 1'b1;
      axi5_cr_ready_i = 1'b1;
      axi5_aw_pending_o = 1'b1;
      axi5_aw_valid_chk_o = 1'b1;
      axi5_cd_valid_o = 1'b1;
      axi5_awakeup_o = 1'b1;
      axi5_varqosaccept_i = 1'b1;
      axi5_syscoreq_o = 1'b1;
      axi5_broadcastatomic_i = 1'b1;
      axi5_activatereq_o = 1'b1;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      axi5_aw_valid_o = 1'b0;
      axi5_aw_ready_i = 1'b0;
      axi5_w_ready_i = 1'b1;
      axi5_b_ready_o = 1'b0;
      axi5_r_ready_o = 1'b0;
      axi5_cr_ready_i = 1'b0;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      axi5_w_valid_o = 1'b0;
      axi5_w_ready_i = 1'b0;
      axi5_b_valid_i = 1'b1;
      axi5_b_ready_o = 1'b1;
      axi5_b_tag_match_i = 2'h2;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      axi5_b_valid_i = 1'b0;
      axi5_b_ready_o = 1'b0;
      axi5_ar_valid_o = 1'b1;
      axi5_ar_ready_i = 1'b1;
      axi5_ar_mecid_o = 16'hb66b;
      axi5_ar_chunken_o = 1'b1;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      axi5_ar_valid_o = 1'b0;
      axi5_ar_ready_i = 1'b0;
      axi5_r_valid_i = 1'b1;
      axi5_r_ready_o = 1'b1;
      axi5_r_chunknum_i = 5'h07;
      axi5_ac_valid_i = 1'b1;
      axi5_ac_addr_i = 32'h12345678;
      axi5_ac_vmidext_i = 4'h9;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      axi5_r_valid_i = 1'b0;
      axi5_r_ready_o = 1'b0;
      axi5_ac_ready_o = 1'b1;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      axi5_ac_valid_i = 1'b0;
      axi5_ac_ready_o = 1'b0;
      axi5_cr_valid_o = 1'b1;
      axi5_cr_ready_i = 1'b1;
      axi5_cr_trace_o = 1'b1;
    end
    #1 clk = 1'b1;
    #0 $finish;
  end
endmodule
