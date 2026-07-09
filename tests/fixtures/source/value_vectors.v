`timescale 1ns/1ns

module top;
  reg clk_r = 1'b0;
  reg [7:0] data_r = 8'b00000000;
  reg [3:0] nibble_r = 4'b1010;
  reg [3:0] status_r = 4'bzzzz;
  reg [0:3] asc_r = 4'b1100;

  wire clk = clk_r;
  wire [7:0] data = data_r;
  wire [3:0] nibble = nibble_r;
  wire [3:0] status = status_r;
  wire [0:3] asc = asc_r;

  initial begin
    $dumpfile("value_vectors.vcd");
    $dumpvars(0, clk, data, nibble, status, asc);
    #5 begin
      clk_r = 1'b1;
      data_r = 8'b00001111;
      nibble_r = 4'b10xz;
      status_r = 4'b0011;
      asc_r = 4'b0011;
    end
    #5 begin
      clk_r = 1'b0;
      data_r = 8'b11110000;
      nibble_r = 4'b0101;
      status_r = 4'bxxxx;
      asc_r = 4'b1010;
    end
    #0 $finish;
  end
endmodule
