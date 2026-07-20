`timescale 1ns/1ns

module top;
  reg clk = 1'b0;
  reg ahb5_h_resetn_i = 1'b0;
  reg [1:0] ahb5_h_trans_o = 2'b00;
  reg ahb5_h_ready_i = 1'b1;
  reg ahb5_h_write_o = 1'b0;
  reg [31:0] ahb5_h_addr_o = 32'h00000000;
  reg [2:0] ahb5_h_burst_o = 3'b000;
  reg ahb5_h_mastlock_o = 1'b0;
  reg [6:0] ahb5_h_prot_o = 7'h03;
  reg [2:0] ahb5_h_size_o = 3'b010;
  reg [3:0] ahb5_h_auser_o = 4'h0;
  reg ahb5_h_nonsec_o = 1'b0;
  reg ahb5_h_excl_o = 1'b0;
  reg [3:0] ahb5_h_master_o = 4'h0;
  reg [31:0] ahb5_h_wdata_o = 32'h00000000;
  reg [3:0] ahb5_h_wstrb_o = 4'h0;
  reg [2:0] ahb5_h_wuser_o = 3'h0;
  reg [31:0] ahb5_h_rdata_i = 32'h00000000;
  reg [2:0] ahb5_h_ruser_i = 3'h0;
  reg [1:0] ahb5_h_buser_i = 2'h0;
  reg ahb5_h_resp_i = 1'b0;
  reg ahb5_h_exokay_i = 1'b0;

  reg ahb5_h_readyout_i = 1'b1;
  reg ahb5_h_sel_i = 1'b1;
  reg ahb5_h_addr_chk_o = 1'b0;
  reg ahb5_h_addr_parity_o = 1'b0;

  initial begin
    $dumpfile("extract_ahb5.vcd");
    $dumpvars(0, top);

    #4;
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 ahb5_h_resetn_i = 1'b1;
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      ahb5_h_trans_o = 2'b10;
      ahb5_h_write_o = 1'b1;
      ahb5_h_addr_o = 32'h80001000;
      ahb5_h_nonsec_o = 1'b1;
      ahb5_h_excl_o = 1'b1;
      ahb5_h_master_o = 4'ha;
      ahb5_h_wdata_o = 32'h11223344;
      ahb5_h_wstrb_o = 4'b0101;
      ahb5_h_wuser_o = 3'h3;
      ahb5_h_buser_i = 2'h2;
      ahb5_h_exokay_i = 1'b1;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      ahb5_h_trans_o = 2'b10;
      ahb5_h_write_o = 1'b0;
      ahb5_h_addr_o = 32'h80002000;
      ahb5_h_nonsec_o = 1'b0;
      ahb5_h_excl_o = 1'b1;
      ahb5_h_master_o = 4'hb;
      ahb5_h_exokay_i = 1'b1;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      ahb5_h_trans_o = 2'b00;
      ahb5_h_rdata_i = 32'h55667788;
      ahb5_h_ruser_i = 3'h6;
      ahb5_h_buser_i = 2'h1;
    end
    #1 clk = 1'b1;
    #0 $finish;
  end
endmodule
