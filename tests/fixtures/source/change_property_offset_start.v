`timescale 1ns/1ns

module top;
  reg valid_r = 1'b1;
  reg ready_r = 1'b1;
  wire valid = valid_r;
  wire ready = ready_r;

  initial begin
    $dumpfile("change_property_offset_start.vcd");
    #100 $dumpvars(0, valid, ready);
    #1 ready_r = 1'b0;
    #4 valid_r = 1'b0;
    #5 valid_r = 1'b1;
    #0 $finish;
  end
endmodule
