`timescale 1ns/1ns

module top;
  reg clk = 1'b0;
  reg ahb_lite_h_resetn_i = 1'b0;
  reg [1:0] ahb_lite_h_trans_o = 2'b00;
  reg ahb_lite_h_ready_i = 1'b1;
  reg ahb_lite_h_write_o = 1'b0;
  reg [31:0] ahb_lite_h_addr_o = 32'h00000000;
  reg [2:0] ahb_lite_h_burst_o = 3'b000;
  reg ahb_lite_h_mastlock_o = 1'b0;
  reg [3:0] ahb_lite_h_prot_o = 4'b0011;
  reg [2:0] ahb_lite_h_size_o = 3'b010;
  reg [3:0] ahb_lite_h_auser_o = 4'h0;
  reg [31:0] ahb_lite_h_wdata_o = 32'h00000000;
  reg [3:0] ahb_lite_h_wstrb_o = 4'h0;
  reg [2:0] ahb_lite_h_wuser_o = 3'h0;
  reg [31:0] ahb_lite_h_rdata_i = 32'h00000000;
  reg [2:0] ahb_lite_h_ruser_i = 3'h0;
  reg [1:0] ahb_lite_h_buser_i = 2'h0;
  reg ahb_lite_h_resp_i = 1'b0;

  initial begin
    $dumpfile("extract_ahb_lite.vcd");
    $dumpvars(0, top);

    #4;
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 ahb_lite_h_resetn_i = 1'b1;
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      ahb_lite_h_trans_o = 2'b10;
      ahb_lite_h_write_o = 1'b0;
      ahb_lite_h_addr_o = 32'h00001000;
      ahb_lite_h_auser_o = 4'h1;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      ahb_lite_h_ready_i = 1'b0;
      ahb_lite_h_rdata_i = 32'h11111111;
      ahb_lite_h_resp_i = 1'b0;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      ahb_lite_h_rdata_i = 32'hdeadbeef;
      ahb_lite_h_resp_i = 1'b1;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      ahb_lite_h_ready_i = 1'b1;
      ahb_lite_h_trans_o = 2'b00;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      ahb_lite_h_resp_i = 1'b0;
      ahb_lite_h_trans_o = 2'b10;
      ahb_lite_h_write_o = 1'b1;
      ahb_lite_h_addr_o = 32'h00002000;
      ahb_lite_h_wdata_o = 32'ha5a55a5a;
      ahb_lite_h_wstrb_o = 4'b0101;
      ahb_lite_h_wuser_o = 3'h5;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      ahb_lite_h_trans_o = 2'b11;
      ahb_lite_h_write_o = 1'b0;
      ahb_lite_h_addr_o = 32'h00002004;
      ahb_lite_h_rdata_i = 32'h12345678;
      ahb_lite_h_ruser_i = 3'h6;
      ahb_lite_h_buser_i = 2'h2;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      ahb_lite_h_trans_o = 2'b01;
      ahb_lite_h_write_o = 1'b1;
      ahb_lite_h_addr_o = 32'h00002008;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      ahb_lite_h_trans_o = 2'b00;
      ahb_lite_h_write_o = 1'b0;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 ahb_lite_h_ready_i = 1'bx;
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3;
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      ahb_lite_h_ready_i = 1'b1;
      ahb_lite_h_trans_o = 2'b10;
      ahb_lite_h_write_o = 1'bx;
      ahb_lite_h_addr_o = 32'h00003000;
      ahb_lite_h_wdata_o = 32'hcafef00d;
      ahb_lite_h_wstrb_o = 4'hf;
      ahb_lite_h_rdata_i = 32'h0badc0de;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 begin
      ahb_lite_h_trans_o = 2'b00;
      ahb_lite_h_resp_i = 1'b0;
    end
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 ahb_lite_h_resetn_i = 1'b0;
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3;
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 ahb_lite_h_resetn_i = 1'bx;
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 ahb_lite_h_resetn_i = 1'b0;
    #1 clk = 1'b1;
    #1 clk = 1'b0;

    #3 ahb_lite_h_resetn_i = 1'b1;
    #1 clk = 1'b1;
    #0 $finish;
  end
endmodule
