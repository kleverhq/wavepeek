`timescale 1ns/1ns

module include_padding;
  parameter INDEX = 0;
  reg [31:0] p00 = INDEX;
  reg [31:0] p01 = INDEX + 1;
  reg [31:0] p02 = INDEX + 2;
  reg [31:0] p03 = INDEX + 3;
  reg [31:0] p04 = INDEX + 4;
  reg [31:0] p05 = INDEX + 5;
  reg [31:0] p06 = INDEX + 6;
  reg [31:0] p07 = INDEX + 7;
  reg [31:0] p08 = INDEX + 8;
  reg [31:0] p09 = INDEX + 9;
  reg [31:0] p10 = INDEX + 10;
  reg [31:0] p11 = INDEX + 11;
  reg [31:0] p12 = INDEX + 12;
  reg [31:0] p13 = INDEX + 13;
  reg [31:0] p14 = INDEX + 14;
  reg [31:0] p15 = INDEX + 15;
endmodule

module top;
  reg aclk = 0;
  reg arvalid = 1;
  reg arready = 1;
  reg [31:0] araddr = 32'h1234;
  genvar i;
  generate
    for (i = 0; i < 2048; i = i + 1) begin: padding
      include_padding #(.INDEX(i)) leaf();
    end
  endgenerate

  initial begin
    $dumpfile("extract_global_include.vcd");
    $dumpvars(0, top);
    #5 aclk = 1;
    #5 aclk = 0;
    #5 $finish;
  end
endmodule
