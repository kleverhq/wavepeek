`timescale 1ns/1ns

module top;
  reg clk_r = 1'b0;
  reg valid_r = 1'b0;
  reg ready_r = 1'b0;
  reg [7:0] data_r = 8'b00000000;
  reg [1:0] state_r = 2'b00;
  reg pulse_r = 1'b0;

  wire clk = clk_r;
  wire valid = valid_r;
  wire ready = ready_r;
  wire [7:0] data = data_r;
  wire [1:0] state = state_r;
  wire pulse = pulse_r;

  initial begin
    $dumpfile("change_property_core.vcd");
    $dumpvars(0, clk, valid, ready, data, state, pulse);
    #5 begin
      clk_r = 1'b1;
      valid_r = 1'b1;
      data_r = 8'b00001111;
      state_r = 2'b01;
    end
    #2 data_r = 8'b00011111;
    #3 begin
      clk_r = 1'b0;
      ready_r = 1'b1;
      pulse_r = 1'b1;
    end
    #5 begin
      clk_r = 1'b1;
      data_r = 8'b00101010;
      state_r = 2'b10;
      pulse_r = 1'b0;
    end
    #5 begin
      clk_r = 1'b0;
      valid_r = 1'b0;
    end
    #5 begin
      clk_r = 1'b1;
      ready_r = 1'b0;
      state_r = 2'b11;
    end
    #5 begin
      clk_r = 1'b0;
      valid_r = 1'b1;
      ready_r = 1'b1;
      data_r = 8'b00111100;
    end
    #5 begin
      clk_r = 1'b1;
      state_r = 2'b00;
    end
    #0 $finish;
  end
endmodule
