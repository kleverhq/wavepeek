`timescale 1ns/1ns

module leaf #(parameter Q = 1);
    reg q = Q;
endmodule

module top;
    leaf \u_reg[0] ();
    leaf #(0) u_reg[0:0] ();
    initial #1 $finish;
endmodule
