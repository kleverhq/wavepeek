`timescale 1ns/1ns

module top;
  reg clk_r = 1'b0;
  reg resetn_r = 1'b1;
  reg psel_r = 1'b0;
  reg penable_r = 1'b0;
  reg pwrite_r = 1'b0;
  reg pready_r = 1'b1;
  reg [7:0] paddr_r = 8'h00;
  reg [2:0] pprot_r = 3'h0;
  reg [7:0] pwdata_r = 8'h00;
  reg pstrb_r = 1'b0;
  reg [7:0] prdata_r = 8'h00;
  reg pslverr_r = 1'b0;

  wire uart_apb_p_clk_i = clk_r;
  wire uart_apb_presetn_i = resetn_r;
  wire uart_apb_psel_o = psel_r;
  wire uart_apb_penable_o = penable_r;
  wire uart_apb_pwrite_o = pwrite_r;
  wire uart_apb_pready_i = pready_r;
  wire [7:0] uart_apb_p_addr_o = paddr_r;
  wire [2:0] uart_apb_pprot_o = pprot_r;
  wire [7:0] uart_apb_pwdata_o = pwdata_r;
  wire uart_apb_pstrb_o = pstrb_r;
  wire [7:0] uart_apb_prdata_i = prdata_r;
  wire uart_apb_pslverr_i = pslverr_r;
  wire uart_apb_preadychk_i = 1'b0;
  wire uart_apb_psel0_o = 1'b0;
  wire uart_apb_pselx_o = 1'b0;
  wire [7:0] uart_apb_shadow_paddr_o = paddr_r;
  wire uart_apb_paddr_pwrite_o = pwrite_r;
  wire uart_apb_misc_o = 1'b0;

  initial begin
    $dumpfile("extract_apb4.vcd");
    $dumpvars(
      0,
      uart_apb_p_clk_i,
      uart_apb_presetn_i,
      uart_apb_psel_o,
      uart_apb_penable_o,
      uart_apb_pwrite_o,
      uart_apb_pready_i,
      uart_apb_p_addr_o,
      uart_apb_pprot_o,
      uart_apb_pwdata_o,
      uart_apb_pstrb_o,
      uart_apb_prdata_i,
      uart_apb_pslverr_i,
      uart_apb_preadychk_i,
      uart_apb_psel0_o,
      uart_apb_pselx_o,
      uart_apb_shadow_paddr_o,
      uart_apb_paddr_pwrite_o,
      uart_apb_misc_o
    );

    #4 begin
      psel_r = 1'b1;
      penable_r = 1'b0;
      pwrite_r = 1'b1;
      paddr_r = 8'h40;
      pprot_r = 3'h2;
      pwdata_r = 8'hde;
      pstrb_r = 1'b1;
    end
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;

    #3 begin
      penable_r = 1'b1;
      pready_r = 1'b0;
    end
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;
    #3;
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;

    #3 begin
      pready_r = 1'b1;
      pslverr_r = 1'b0;
      prdata_r = 8'hff;
    end
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;

    #3 begin
      penable_r = 1'b0;
      pwrite_r = 1'b0;
      paddr_r = 8'h44;
      pprot_r = 3'h1;
      pstrb_r = 1'b0;
      prdata_r = 8'ha5;
    end
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;

    #3 begin
      penable_r = 1'b1;
      pslverr_r = 1'b1;
    end
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;

    #3 begin
      pwrite_r = 1'b1;
      paddr_r = 8'h48;
      pwdata_r = 8'h5a;
      pslverr_r = 1'b0;
    end
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;

    #3 begin
      penable_r = 1'b0;
      pready_r = 1'bx;
      pwrite_r = 1'bx;
      paddr_r = 8'h4c;
      pwdata_r = 8'hcc;
    end
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;

    #3 penable_r = 1'b1;
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;

    #3 begin
      resetn_r = 1'b0;
      penable_r = 1'b0;
      pready_r = 1'b1;
      pwrite_r = 1'b0;
    end
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;

    #3 begin
      resetn_r = 1'b1;
      psel_r = 1'bx;
      penable_r = 1'b0;
    end
    #1 clk_r = 1'b1;
    #1 clk_r = 1'b0;

    #3 begin
      psel_r = 1'b1;
      penable_r = 1'bx;
    end
    #1 clk_r = 1'b1;
    #0 $finish;
  end
endmodule
