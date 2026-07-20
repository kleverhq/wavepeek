`timescale 1ns/1ns

module top;
  reg clk_r = 1'b0;
  reg rst_n_r = 1'b0;

  reg s_axis_t_valid_o_r = 1'b1;
  reg s_axis_tready_i_r = 1'b1;
  reg [31:0] s_axis_t_data_o_r = 32'h11111111;
  reg [3:0] s_axis_tstrb_o_r = 4'hf;
  reg [3:0] s_axis_t_keep_o_r = 4'hf;
  reg s_axis_tlast_o_r = 1'b0;
  reg [2:0] s_axis_tid_o_r = 3'h1;
  reg [1:0] s_axis_t_dest_o_r = 2'h2;
  reg [7:0] s_axis_tuser_o_r = 8'ha5;

  reg m_axis_tvalid_o_r = 1'b1;
  reg [15:0] m_axis_tdata_o_r = 16'h1001;
  reg m_axis_tlast_o_r = 1'b0;

  wire clk = clk_r;
  wire rst_n = rst_n_r;
  wire s_axis_t_valid_o = s_axis_t_valid_o_r;
  wire s_axis_tready_i = s_axis_tready_i_r;
  wire [31:0] s_axis_t_data_o = s_axis_t_data_o_r;
  wire [3:0] s_axis_tstrb_o = s_axis_tstrb_o_r;
  wire [3:0] s_axis_t_keep_o = s_axis_t_keep_o_r;
  wire s_axis_tlast_o = s_axis_tlast_o_r;
  wire [2:0] s_axis_tid_o = s_axis_tid_o_r;
  wire [1:0] s_axis_t_dest_o = s_axis_t_dest_o_r;
  wire [7:0] s_axis_tuser_o = s_axis_tuser_o_r;
  wire m_axis_tvalid_o = m_axis_tvalid_o_r;
  wire [15:0] m_axis_tdata_o = m_axis_tdata_o_r;
  wire m_axis_tlast_o = m_axis_tlast_o_r;

  wire s_axis_tvalidchk_o = s_axis_t_valid_o_r;
  wire s_axis_t_ready_chk_i = s_axis_tready_i_r;
  wire [3:0] s_axis_tdatachk_o = s_axis_t_data_o_r[3:0];
  wire s_axis_twakeup_o = s_axis_t_valid_o_r;

  initial begin
    $dumpfile("extract_axistream.vcd");
    $dumpvars(
      0,
      clk,
      rst_n,
      s_axis_t_valid_o,
      s_axis_tready_i,
      s_axis_t_data_o,
      s_axis_tstrb_o,
      s_axis_t_keep_o,
      s_axis_tlast_o,
      s_axis_tid_o,
      s_axis_t_dest_o,
      s_axis_tuser_o,
      m_axis_tvalid_o,
      m_axis_tdata_o,
      m_axis_tlast_o,
      s_axis_tvalidchk_o,
      s_axis_t_ready_chk_i,
      s_axis_tdatachk_o,
      s_axis_twakeup_o
    );

    #4 s_axis_t_data_o_r = 32'h22222222;
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;

    #3 begin
      rst_n_r = 1'b1;
      s_axis_t_data_o_r = 32'h33333333;
      m_axis_tdata_o_r = 16'h2002;
    end
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;

    #3 begin
      s_axis_tready_i_r = 1'b0;
      s_axis_t_data_o_r = 32'h44444444;
      m_axis_tdata_o_r = 16'h3003;
    end
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;

    #3 begin
      s_axis_tready_i_r = 1'b1;
      s_axis_t_data_o_r = 32'hdeadbeef;
      s_axis_tlast_o_r = 1'b1;
      m_axis_tdata_o_r = 16'hbeef;
      m_axis_tlast_o_r = 1'b1;
    end
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;

    #3 begin
      s_axis_t_data_o_r = 32'hdeadbeef;
      m_axis_tdata_o_r = 16'hbeef;
    end
    #1 clk_r = 1'b1;
    #0 $finish;
  end
endmodule
