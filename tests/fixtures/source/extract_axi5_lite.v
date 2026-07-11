`timescale 1ns/1ns

module top;
  reg clk = 1'b0;

  reg axi5_lite_aw_valid_o = 1'b0;
  reg axi5_lite_aw_ready_i = 1'b0;
  reg [3:0] axi5_lite_aw_id_o = 4'h0;
  reg [7:0] axi5_lite_aw_addr_o = 8'h00;
  reg [2:0] axi5_lite_aw_size_o = 3'h0;
  reg axi5_lite_aw_id_unq_o = 1'b0;

  reg axi5_lite_w_valid_o = 1'b0;
  reg axi5_lite_w_ready_i = 1'b0;
  reg [7:0] axi5_lite_w_data_o = 8'h00;
  reg axi5_lite_w_poison_o = 1'b0;

  reg axi5_lite_b_valid_i = 1'b0;
  reg axi5_lite_b_ready_o = 1'b0;
  reg [1:0] axi5_lite_b_resp_i = 2'h0;

  reg axi5_lite_ar_valid_o = 1'b0;
  reg axi5_lite_ar_ready_i = 1'b0;
  reg [3:0] axi5_lite_ar_id_o = 4'h0;
  reg [7:0] axi5_lite_ar_addr_o = 8'h00;
  reg [2:0] axi5_lite_ar_size_o = 3'h0;
  reg axi5_lite_ar_id_unq_o = 1'b0;

  reg axi5_lite_r_valid_i = 1'b0;
  reg axi5_lite_r_ready_o = 1'b0;
  reg [7:0] axi5_lite_r_data_i = 8'h00;
  reg axi5_lite_r_poison_i = 1'b0;

  reg axi5_lite_w_last_o = 1'b0;
  reg axi5_lite_r_last_i = 1'b0;
  reg axi5_lite_ac_valid_i = 1'b0;
  reg axi5_lite_aw_pending_o = 1'b0;
  reg axi5_lite_aw_valid_chk_o = 1'b0;

  initial begin
    $dumpfile("extract_axi5_lite.vcd");
    $dumpvars(0, top);

    #4 begin
      axi5_lite_aw_valid_o = 1'b1;
      axi5_lite_aw_ready_i = 1'b1;
      axi5_lite_aw_id_o = 4'h1;
      axi5_lite_aw_addr_o = 8'h24;
      axi5_lite_aw_size_o = 3'h0;
      axi5_lite_aw_id_unq_o = 1'b1;
      axi5_lite_w_valid_o = 1'b1;
      axi5_lite_w_data_o = 8'ha5;
      axi5_lite_w_poison_o = 1'b1;
      axi5_lite_b_ready_o = 1'b1;
      axi5_lite_r_ready_o = 1'b1;
      axi5_lite_w_last_o = 1'b1;
      axi5_lite_r_last_i = 1'b1;
      axi5_lite_ac_valid_i = 1'b1;
      axi5_lite_aw_pending_o = 1'b1;
      axi5_lite_aw_valid_chk_o = 1'b1;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      axi5_lite_aw_valid_o = 1'b0;
      axi5_lite_aw_ready_i = 1'b0;
      axi5_lite_w_ready_i = 1'b1;
      axi5_lite_b_ready_o = 1'b0;
      axi5_lite_r_ready_o = 1'b0;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      axi5_lite_w_valid_o = 1'b0;
      axi5_lite_w_ready_i = 1'b0;
      axi5_lite_b_valid_i = 1'b1;
      axi5_lite_b_ready_o = 1'b1;
      axi5_lite_b_resp_i = 2'h2;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      axi5_lite_b_valid_i = 1'b0;
      axi5_lite_b_ready_o = 1'b0;
      axi5_lite_ar_valid_o = 1'b1;
      axi5_lite_ar_ready_i = 1'b1;
      axi5_lite_ar_id_o = 4'h2;
      axi5_lite_ar_addr_o = 8'h48;
      axi5_lite_ar_size_o = 3'h0;
      axi5_lite_ar_id_unq_o = 1'b1;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      axi5_lite_ar_valid_o = 1'b0;
      axi5_lite_ar_ready_i = 1'b0;
      axi5_lite_r_valid_i = 1'b1;
      axi5_lite_r_ready_o = 1'b1;
      axi5_lite_r_data_i = 8'h5a;
      axi5_lite_r_poison_i = 1'b1;
    end
    #1 clk = 1'b1;
    #0 $finish;
  end
endmodule
