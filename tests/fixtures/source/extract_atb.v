`timescale 1ns/1ns

module top;
  reg atclk_r = 1'b0;
  reg atresetn_r = 1'b0;
  reg atvalid_r = 1'b1;
  reg atready_r = 1'b1;
  reg [1:0] atbytes_r = 2'h0;
  reg [31:0] atdata_r = 32'h00000000;
  reg [7:0] atdata8_r = 8'h00;
  reg [6:0] atid_r = 7'h01;
  reg afvalid_r = 1'b1;
  reg afready_r = 1'b1;
  reg syncreq_r = 1'b1;

  wire trace_at_clk_i = atclk_r;
  wire trace_at_reset_n_i = atresetn_r;
  wire trace_at_valid_o = atvalid_r;
  wire trace_at_ready_i = atready_r;
  wire [1:0] trace_at_bytes_o = atbytes_r;
  wire [31:0] trace_at_data_o = atdata_r;
  wire [7:0] trace_at_data8_o = atdata8_r;
  wire [6:0] trace_at_id_o = atid_r;
  wire trace_af_valid_i = afvalid_r;
  wire trace_af_ready_o = afready_r;
  wire trace_sync_req_i = syncreq_r;
  wire trace_at_clken_o = 1'b1;
  wire trace_at_wakeup_o = 1'b1;
  wire trace_at_valid_chk_o = 1'b1;
  wire trace_at_ready_check_o = 1'b1;
  wire other_at_valid_o = atvalid_r;

  initial begin
    $dumpfile("extract_atb.vcd");
    $dumpvars(
      0,
      trace_at_clk_i,
      trace_at_reset_n_i,
      trace_at_valid_o,
      trace_at_ready_i,
      trace_at_bytes_o,
      trace_at_data_o,
      trace_at_data8_o,
      trace_at_id_o,
      trace_af_valid_i,
      trace_af_ready_o,
      trace_sync_req_i,
      trace_at_clken_o,
      trace_at_wakeup_o,
      trace_at_valid_chk_o,
      trace_at_ready_check_o,
      other_at_valid_o
    );

    // Reset-low edge: all conditions are true but every event is suppressed.
    #5 atclk_r = 1'b1;
    #1 atclk_r = 1'b0;

    // Stalled transfer and pending flush do not emit events.
    #3 begin
      atresetn_r = 1'b1;
      atready_r = 1'b0;
      afready_r = 1'b0;
      syncreq_r = 1'b0;
    end
    #1 atclk_r = 1'b1;
    #1 atclk_r = 1'b0;

    // All three independent events occur on one edge.
    #3 begin
      atready_r = 1'b1;
      atbytes_r = 2'h3;
      atdata_r = 32'h44332211;
      atdata8_r = 8'ha5;
      atid_r = 7'h10;
      afready_r = 1'b1;
      syncreq_r = 1'b1;
    end
    #1 atclk_r = 1'b1;
    #1 atclk_r = 1'b0;

    // Identical sampled levels are separate stateless events.
    #4 atclk_r = 1'b1;
    #1 atclk_r = 1'b0;

    // Unknown payload bits are preserved on an accepted transfer.
    #3 begin
      atdata_r = 32'hxx332211;
      atdata8_r = 8'hxx;
      syncreq_r = 1'b0;
      afready_r = 1'b0;
    end
    #1 atclk_r = 1'b1;
    #1 atclk_r = 1'b0;

    // ATID 0x7d remains an ordinary accepted transfer observation.
    #3 begin
      atbytes_r = 2'h0;
      atdata_r = 32'h00000010;
      atdata8_r = 8'h10;
      atid_r = 7'h7d;
      afvalid_r = 1'b0;
      syncreq_r = 1'b1;
    end
    #1 atclk_r = 1'b1;
    #1 atclk_r = 1'b0;

    // Unknown reset suppresses every otherwise true event locally.
    #3 begin
      atresetn_r = 1'bx;
      afvalid_r = 1'b1;
      afready_r = 1'b1;
    end
    #1 atclk_r = 1'b1;
    #1 atclk_r = 1'b0;

    // A final flush-only edge proves independent channel extraction.
    #3 begin
      atresetn_r = 1'b1;
      atvalid_r = 1'b0;
      syncreq_r = 1'b0;
    end
    #1 atclk_r = 1'b1;
    #0 $finish;
  end
endmodule
