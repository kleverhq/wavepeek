use super::{parse_event_expr_ast, parse_logical_expr_ast};

#[test]
fn parser_smoke_logical_001() {
    parse_logical_expr_ast("sig1 + 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_002() {
    parse_logical_expr_ast("(sig2 & mask2) == 2").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_003() {
    parse_logical_expr_ast("sig3[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_004() {
    parse_logical_expr_ast("sig4[3:0] != 4'hf").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_005() {
    parse_logical_expr_ast("sig5 inside {1, 2, 3}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_006() {
    parse_logical_expr_ast("flag6 ? yes6 : no6").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_007() {
    parse_logical_expr_ast("{2{sig7}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_008() {
    parse_logical_expr_ast("sig8 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_009() {
    parse_logical_expr_ast("sig9 >>> 2").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_010() {
    parse_logical_expr_ast("~&sig10").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_011() {
    parse_logical_expr_ast("^~sig11").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_012() {
    parse_logical_expr_ast("type(state12)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_013() {
    parse_logical_expr_ast("logic[8]'(sig13)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_014() {
    parse_logical_expr_ast("unsigned'(sig14)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_015() {
    parse_logical_expr_ast("sig15.triggered()").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_016() {
    parse_logical_expr_ast("sig16 + 16").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_017() {
    parse_logical_expr_ast("(sig17 & mask17) == 17")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_018() {
    parse_logical_expr_ast("sig18[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_019() {
    parse_logical_expr_ast("sig19[3:0] != 4'hf").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_020() {
    parse_logical_expr_ast("sig20 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_021() {
    parse_logical_expr_ast("flag21 ? yes21 : no21").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_022() {
    parse_logical_expr_ast("{2{sig22}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_023() {
    parse_logical_expr_ast("sig23 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_024() {
    parse_logical_expr_ast("sig24 >>> 2").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_025() {
    parse_logical_expr_ast("~&sig25").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_026() {
    parse_logical_expr_ast("^~sig26").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_027() {
    parse_logical_expr_ast("type(state27)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_028() {
    parse_logical_expr_ast("logic[8]'(sig28)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_029() {
    parse_logical_expr_ast("unsigned'(sig29)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_030() {
    parse_logical_expr_ast("sig30.triggered()").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_031() {
    parse_logical_expr_ast("sig31 + 31").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_032() {
    parse_logical_expr_ast("(sig32 & mask32) == 32")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_033() {
    parse_logical_expr_ast("sig33[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_034() {
    parse_logical_expr_ast("sig34[3:0] != 4'hf").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_035() {
    parse_logical_expr_ast("sig35 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_036() {
    parse_logical_expr_ast("flag36 ? yes36 : no36").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_037() {
    parse_logical_expr_ast("{2{sig37}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_038() {
    parse_logical_expr_ast("sig38 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_039() {
    parse_logical_expr_ast("sig39 >>> 2").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_040() {
    parse_logical_expr_ast("~&sig40").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_041() {
    parse_logical_expr_ast("^~sig41").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_042() {
    parse_logical_expr_ast("type(state42)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_043() {
    parse_logical_expr_ast("logic[8]'(sig43)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_044() {
    parse_logical_expr_ast("unsigned'(sig44)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_045() {
    parse_logical_expr_ast("sig45.triggered()").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_046() {
    parse_logical_expr_ast("sig46 + 46").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_047() {
    parse_logical_expr_ast("(sig47 & mask47) == 47")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_048() {
    parse_logical_expr_ast("sig48[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_049() {
    parse_logical_expr_ast("sig49[3:0] != 4'hf").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_050() {
    parse_logical_expr_ast("sig50 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_051() {
    parse_logical_expr_ast("flag51 ? yes51 : no51").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_052() {
    parse_logical_expr_ast("{2{sig52}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_053() {
    parse_logical_expr_ast("sig53 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_054() {
    parse_logical_expr_ast("sig54 >>> 2").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_055() {
    parse_logical_expr_ast("~&sig55").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_056() {
    parse_logical_expr_ast("^~sig56").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_057() {
    parse_logical_expr_ast("type(state57)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_058() {
    parse_logical_expr_ast("logic[8]'(sig58)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_059() {
    parse_logical_expr_ast("unsigned'(sig59)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_060() {
    parse_logical_expr_ast("sig60.triggered()").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_061() {
    parse_logical_expr_ast("sig61 + 61").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_062() {
    parse_logical_expr_ast("(sig62 & mask62) == 62")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_063() {
    parse_logical_expr_ast("sig63[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_064() {
    parse_logical_expr_ast("sig64[3:0] != 4'hf").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_065() {
    parse_logical_expr_ast("sig65 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_066() {
    parse_logical_expr_ast("flag66 ? yes66 : no66").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_067() {
    parse_logical_expr_ast("{2{sig67}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_068() {
    parse_logical_expr_ast("sig68 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_069() {
    parse_logical_expr_ast("sig69 >>> 2").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_070() {
    parse_logical_expr_ast("~&sig70").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_071() {
    parse_logical_expr_ast("^~sig71").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_072() {
    parse_logical_expr_ast("type(state72)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_073() {
    parse_logical_expr_ast("logic[8]'(sig73)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_074() {
    parse_logical_expr_ast("unsigned'(sig74)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_075() {
    parse_logical_expr_ast("sig75.triggered()").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_076() {
    parse_logical_expr_ast("sig76 + 76").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_077() {
    parse_logical_expr_ast("(sig77 & mask77) == 77")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_078() {
    parse_logical_expr_ast("sig78[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_079() {
    parse_logical_expr_ast("sig79[3:0] != 4'hf").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_080() {
    parse_logical_expr_ast("sig80 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_081() {
    parse_logical_expr_ast("flag81 ? yes81 : no81").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_082() {
    parse_logical_expr_ast("{2{sig82}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_083() {
    parse_logical_expr_ast("sig83 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_084() {
    parse_logical_expr_ast("sig84 >>> 2").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_085() {
    parse_logical_expr_ast("~&sig85").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_086() {
    parse_logical_expr_ast("^~sig86").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_087() {
    parse_logical_expr_ast("type(state87)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_088() {
    parse_logical_expr_ast("logic[8]'(sig88)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_089() {
    parse_logical_expr_ast("unsigned'(sig89)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_090() {
    parse_logical_expr_ast("sig90.triggered()").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_091() {
    parse_logical_expr_ast("sig91 + 91").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_092() {
    parse_logical_expr_ast("(sig92 & mask92) == 92")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_093() {
    parse_logical_expr_ast("sig93[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_094() {
    parse_logical_expr_ast("sig94[3:0] != 4'hf").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_095() {
    parse_logical_expr_ast("sig95 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_096() {
    parse_logical_expr_ast("flag96 ? yes96 : no96").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_097() {
    parse_logical_expr_ast("{2{sig97}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_098() {
    parse_logical_expr_ast("sig98 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_099() {
    parse_logical_expr_ast("sig99 >>> 2").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_100() {
    parse_logical_expr_ast("~&sig100").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_event_001() {
    parse_event_expr_ast("posedge clk1").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_002() {
    parse_event_expr_ast("negedge rst2 iff enable2").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_003() {
    parse_event_expr_ast("event3 or posedge clk3").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_004() {
    parse_event_expr_ast("a4, b4, c4").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_005() {
    parse_event_expr_ast("posedge clk5").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_006() {
    parse_event_expr_ast("negedge rst6 iff enable6").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_007() {
    parse_event_expr_ast("event7 or posedge clk7").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_008() {
    parse_event_expr_ast("a8, b8, c8").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_009() {
    parse_event_expr_ast("posedge clk9").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_010() {
    parse_event_expr_ast("negedge rst10 iff enable10")
        .expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_011() {
    parse_event_expr_ast("event11 or posedge clk11").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_012() {
    parse_event_expr_ast("a12, b12, c12").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_013() {
    parse_event_expr_ast("posedge clk13").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_014() {
    parse_event_expr_ast("negedge rst14 iff enable14")
        .expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_015() {
    parse_event_expr_ast("event15 or posedge clk15").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_016() {
    parse_event_expr_ast("a16, b16, c16").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_017() {
    parse_event_expr_ast("posedge clk17").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_018() {
    parse_event_expr_ast("negedge rst18 iff enable18")
        .expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_019() {
    parse_event_expr_ast("event19 or posedge clk19").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_020() {
    parse_event_expr_ast("a20, b20, c20").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_021() {
    parse_event_expr_ast("posedge clk21").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_022() {
    parse_event_expr_ast("negedge rst22 iff enable22")
        .expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_023() {
    parse_event_expr_ast("event23 or posedge clk23").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_024() {
    parse_event_expr_ast("a24, b24, c24").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_025() {
    parse_event_expr_ast("posedge clk25").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_026() {
    parse_event_expr_ast("negedge rst26 iff enable26")
        .expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_027() {
    parse_event_expr_ast("event27 or posedge clk27").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_028() {
    parse_event_expr_ast("a28, b28, c28").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_029() {
    parse_event_expr_ast("posedge clk29").expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_event_030() {
    parse_event_expr_ast("negedge rst30 iff enable30")
        .expect("event smoke expression should parse");
}

#[test]
fn parser_smoke_logical_101() {
    parse_logical_expr_ast("(sig101 & mask101) == 5")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_102() {
    parse_logical_expr_ast("sig102[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_103() {
    parse_logical_expr_ast("sig103 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_104() {
    parse_logical_expr_ast("flag104 ? yes104 : no104")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_105() {
    parse_logical_expr_ast("{2{sig105}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_106() {
    parse_logical_expr_ast("signed'(sig106)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_107() {
    parse_logical_expr_ast("sig107 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_108() {
    parse_logical_expr_ast("~|sig108").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_109() {
    parse_logical_expr_ast("type(state109)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_110() {
    parse_logical_expr_ast("sig110 + 110").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_111() {
    parse_logical_expr_ast("(sig111 & mask111) == 15")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_112() {
    parse_logical_expr_ast("sig112[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_113() {
    parse_logical_expr_ast("sig113 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_114() {
    parse_logical_expr_ast("flag114 ? yes114 : no114")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_115() {
    parse_logical_expr_ast("{2{sig115}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_116() {
    parse_logical_expr_ast("signed'(sig116)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_117() {
    parse_logical_expr_ast("sig117 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_118() {
    parse_logical_expr_ast("~|sig118").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_119() {
    parse_logical_expr_ast("type(state119)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_120() {
    parse_logical_expr_ast("sig120 + 120").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_121() {
    parse_logical_expr_ast("(sig121 & mask121) == 9")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_122() {
    parse_logical_expr_ast("sig122[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_123() {
    parse_logical_expr_ast("sig123 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_124() {
    parse_logical_expr_ast("flag124 ? yes124 : no124")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_125() {
    parse_logical_expr_ast("{2{sig125}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_126() {
    parse_logical_expr_ast("signed'(sig126)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_127() {
    parse_logical_expr_ast("sig127 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_128() {
    parse_logical_expr_ast("~|sig128").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_129() {
    parse_logical_expr_ast("type(state129)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_130() {
    parse_logical_expr_ast("sig130 + 130").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_131() {
    parse_logical_expr_ast("(sig131 & mask131) == 3")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_132() {
    parse_logical_expr_ast("sig132[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_133() {
    parse_logical_expr_ast("sig133 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_134() {
    parse_logical_expr_ast("flag134 ? yes134 : no134")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_135() {
    parse_logical_expr_ast("{2{sig135}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_136() {
    parse_logical_expr_ast("signed'(sig136)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_137() {
    parse_logical_expr_ast("sig137 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_138() {
    parse_logical_expr_ast("~|sig138").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_139() {
    parse_logical_expr_ast("type(state139)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_140() {
    parse_logical_expr_ast("sig140 + 140").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_141() {
    parse_logical_expr_ast("(sig141 & mask141) == 13")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_142() {
    parse_logical_expr_ast("sig142[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_143() {
    parse_logical_expr_ast("sig143 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_144() {
    parse_logical_expr_ast("flag144 ? yes144 : no144")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_145() {
    parse_logical_expr_ast("{2{sig145}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_146() {
    parse_logical_expr_ast("signed'(sig146)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_147() {
    parse_logical_expr_ast("sig147 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_148() {
    parse_logical_expr_ast("~|sig148").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_149() {
    parse_logical_expr_ast("type(state149)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_150() {
    parse_logical_expr_ast("sig150 + 150").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_151() {
    parse_logical_expr_ast("(sig151 & mask151) == 7")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_152() {
    parse_logical_expr_ast("sig152[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_153() {
    parse_logical_expr_ast("sig153 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_154() {
    parse_logical_expr_ast("flag154 ? yes154 : no154")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_155() {
    parse_logical_expr_ast("{2{sig155}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_156() {
    parse_logical_expr_ast("signed'(sig156)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_157() {
    parse_logical_expr_ast("sig157 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_158() {
    parse_logical_expr_ast("~|sig158").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_159() {
    parse_logical_expr_ast("type(state159)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_160() {
    parse_logical_expr_ast("sig160 + 160").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_161() {
    parse_logical_expr_ast("(sig161 & mask161) == 1")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_162() {
    parse_logical_expr_ast("sig162[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_163() {
    parse_logical_expr_ast("sig163 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_164() {
    parse_logical_expr_ast("flag164 ? yes164 : no164")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_165() {
    parse_logical_expr_ast("{2{sig165}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_166() {
    parse_logical_expr_ast("signed'(sig166)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_167() {
    parse_logical_expr_ast("sig167 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_168() {
    parse_logical_expr_ast("~|sig168").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_169() {
    parse_logical_expr_ast("type(state169)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_170() {
    parse_logical_expr_ast("sig170 + 170").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_171() {
    parse_logical_expr_ast("(sig171 & mask171) == 11")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_172() {
    parse_logical_expr_ast("sig172[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_173() {
    parse_logical_expr_ast("sig173 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_174() {
    parse_logical_expr_ast("flag174 ? yes174 : no174")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_175() {
    parse_logical_expr_ast("{2{sig175}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_176() {
    parse_logical_expr_ast("signed'(sig176)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_177() {
    parse_logical_expr_ast("sig177 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_178() {
    parse_logical_expr_ast("~|sig178").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_179() {
    parse_logical_expr_ast("type(state179)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_180() {
    parse_logical_expr_ast("sig180 + 180").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_181() {
    parse_logical_expr_ast("(sig181 & mask181) == 5")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_182() {
    parse_logical_expr_ast("sig182[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_183() {
    parse_logical_expr_ast("sig183 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_184() {
    parse_logical_expr_ast("flag184 ? yes184 : no184")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_185() {
    parse_logical_expr_ast("{2{sig185}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_186() {
    parse_logical_expr_ast("signed'(sig186)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_187() {
    parse_logical_expr_ast("sig187 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_188() {
    parse_logical_expr_ast("~|sig188").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_189() {
    parse_logical_expr_ast("type(state189)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_190() {
    parse_logical_expr_ast("sig190 + 190").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_191() {
    parse_logical_expr_ast("(sig191 & mask191) == 15")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_192() {
    parse_logical_expr_ast("sig192[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_193() {
    parse_logical_expr_ast("sig193 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_194() {
    parse_logical_expr_ast("flag194 ? yes194 : no194")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_195() {
    parse_logical_expr_ast("{2{sig195}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_196() {
    parse_logical_expr_ast("signed'(sig196)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_197() {
    parse_logical_expr_ast("sig197 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_198() {
    parse_logical_expr_ast("~|sig198").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_199() {
    parse_logical_expr_ast("type(state199)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_200() {
    parse_logical_expr_ast("sig200 + 200").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_201() {
    parse_logical_expr_ast("(sig201 & mask201) == 9")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_202() {
    parse_logical_expr_ast("sig202[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_203() {
    parse_logical_expr_ast("sig203 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_204() {
    parse_logical_expr_ast("flag204 ? yes204 : no204")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_205() {
    parse_logical_expr_ast("{2{sig205}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_206() {
    parse_logical_expr_ast("signed'(sig206)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_207() {
    parse_logical_expr_ast("sig207 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_208() {
    parse_logical_expr_ast("~|sig208").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_209() {
    parse_logical_expr_ast("type(state209)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_210() {
    parse_logical_expr_ast("sig210 + 210").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_211() {
    parse_logical_expr_ast("(sig211 & mask211) == 3")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_212() {
    parse_logical_expr_ast("sig212[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_213() {
    parse_logical_expr_ast("sig213 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_214() {
    parse_logical_expr_ast("flag214 ? yes214 : no214")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_215() {
    parse_logical_expr_ast("{2{sig215}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_216() {
    parse_logical_expr_ast("signed'(sig216)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_217() {
    parse_logical_expr_ast("sig217 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_218() {
    parse_logical_expr_ast("~|sig218").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_219() {
    parse_logical_expr_ast("type(state219)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_220() {
    parse_logical_expr_ast("sig220 + 220").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_221() {
    parse_logical_expr_ast("(sig221 & mask221) == 13")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_222() {
    parse_logical_expr_ast("sig222[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_223() {
    parse_logical_expr_ast("sig223 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_224() {
    parse_logical_expr_ast("flag224 ? yes224 : no224")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_225() {
    parse_logical_expr_ast("{2{sig225}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_226() {
    parse_logical_expr_ast("signed'(sig226)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_227() {
    parse_logical_expr_ast("sig227 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_228() {
    parse_logical_expr_ast("~|sig228").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_229() {
    parse_logical_expr_ast("type(state229)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_230() {
    parse_logical_expr_ast("sig230 + 230").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_231() {
    parse_logical_expr_ast("(sig231 & mask231) == 7")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_232() {
    parse_logical_expr_ast("sig232[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_233() {
    parse_logical_expr_ast("sig233 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_234() {
    parse_logical_expr_ast("flag234 ? yes234 : no234")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_235() {
    parse_logical_expr_ast("{2{sig235}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_236() {
    parse_logical_expr_ast("signed'(sig236)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_237() {
    parse_logical_expr_ast("sig237 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_238() {
    parse_logical_expr_ast("~|sig238").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_239() {
    parse_logical_expr_ast("type(state239)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_240() {
    parse_logical_expr_ast("sig240 + 240").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_241() {
    parse_logical_expr_ast("(sig241 & mask241) == 1")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_242() {
    parse_logical_expr_ast("sig242[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_243() {
    parse_logical_expr_ast("sig243 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_244() {
    parse_logical_expr_ast("flag244 ? yes244 : no244")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_245() {
    parse_logical_expr_ast("{2{sig245}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_246() {
    parse_logical_expr_ast("signed'(sig246)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_247() {
    parse_logical_expr_ast("sig247 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_248() {
    parse_logical_expr_ast("~|sig248").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_249() {
    parse_logical_expr_ast("type(state249)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_250() {
    parse_logical_expr_ast("sig250 + 250").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_251() {
    parse_logical_expr_ast("(sig251 & mask251) == 11")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_252() {
    parse_logical_expr_ast("sig252[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_253() {
    parse_logical_expr_ast("sig253 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_254() {
    parse_logical_expr_ast("flag254 ? yes254 : no254")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_255() {
    parse_logical_expr_ast("{2{sig255}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_256() {
    parse_logical_expr_ast("signed'(sig256)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_257() {
    parse_logical_expr_ast("sig257 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_258() {
    parse_logical_expr_ast("~|sig258").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_259() {
    parse_logical_expr_ast("type(state259)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_260() {
    parse_logical_expr_ast("sig260 + 260").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_261() {
    parse_logical_expr_ast("(sig261 & mask261) == 5")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_262() {
    parse_logical_expr_ast("sig262[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_263() {
    parse_logical_expr_ast("sig263 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_264() {
    parse_logical_expr_ast("flag264 ? yes264 : no264")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_265() {
    parse_logical_expr_ast("{2{sig265}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_266() {
    parse_logical_expr_ast("signed'(sig266)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_267() {
    parse_logical_expr_ast("sig267 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_268() {
    parse_logical_expr_ast("~|sig268").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_269() {
    parse_logical_expr_ast("type(state269)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_270() {
    parse_logical_expr_ast("sig270 + 270").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_271() {
    parse_logical_expr_ast("(sig271 & mask271) == 15")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_272() {
    parse_logical_expr_ast("sig272[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_273() {
    parse_logical_expr_ast("sig273 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_274() {
    parse_logical_expr_ast("flag274 ? yes274 : no274")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_275() {
    parse_logical_expr_ast("{2{sig275}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_276() {
    parse_logical_expr_ast("signed'(sig276)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_277() {
    parse_logical_expr_ast("sig277 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_278() {
    parse_logical_expr_ast("~|sig278").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_279() {
    parse_logical_expr_ast("type(state279)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_280() {
    parse_logical_expr_ast("sig280 + 280").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_281() {
    parse_logical_expr_ast("(sig281 & mask281) == 9")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_282() {
    parse_logical_expr_ast("sig282[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_283() {
    parse_logical_expr_ast("sig283 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_284() {
    parse_logical_expr_ast("flag284 ? yes284 : no284")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_285() {
    parse_logical_expr_ast("{2{sig285}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_286() {
    parse_logical_expr_ast("signed'(sig286)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_287() {
    parse_logical_expr_ast("sig287 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_288() {
    parse_logical_expr_ast("~|sig288").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_289() {
    parse_logical_expr_ast("type(state289)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_290() {
    parse_logical_expr_ast("sig290 + 290").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_291() {
    parse_logical_expr_ast("(sig291 & mask291) == 3")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_292() {
    parse_logical_expr_ast("sig292[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_293() {
    parse_logical_expr_ast("sig293 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_294() {
    parse_logical_expr_ast("flag294 ? yes294 : no294")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_295() {
    parse_logical_expr_ast("{2{sig295}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_296() {
    parse_logical_expr_ast("signed'(sig296)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_297() {
    parse_logical_expr_ast("sig297 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_298() {
    parse_logical_expr_ast("~|sig298").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_299() {
    parse_logical_expr_ast("type(state299)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_300() {
    parse_logical_expr_ast("sig300 + 300").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_301() {
    parse_logical_expr_ast("(sig301 & mask301) == 13")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_302() {
    parse_logical_expr_ast("sig302[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_303() {
    parse_logical_expr_ast("sig303 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_304() {
    parse_logical_expr_ast("flag304 ? yes304 : no304")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_305() {
    parse_logical_expr_ast("{2{sig305}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_306() {
    parse_logical_expr_ast("signed'(sig306)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_307() {
    parse_logical_expr_ast("sig307 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_308() {
    parse_logical_expr_ast("~|sig308").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_309() {
    parse_logical_expr_ast("type(state309)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_310() {
    parse_logical_expr_ast("sig310 + 310").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_311() {
    parse_logical_expr_ast("(sig311 & mask311) == 7")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_312() {
    parse_logical_expr_ast("sig312[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_313() {
    parse_logical_expr_ast("sig313 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_314() {
    parse_logical_expr_ast("flag314 ? yes314 : no314")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_315() {
    parse_logical_expr_ast("{2{sig315}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_316() {
    parse_logical_expr_ast("signed'(sig316)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_317() {
    parse_logical_expr_ast("sig317 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_318() {
    parse_logical_expr_ast("~|sig318").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_319() {
    parse_logical_expr_ast("type(state319)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_320() {
    parse_logical_expr_ast("sig320 + 320").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_321() {
    parse_logical_expr_ast("(sig321 & mask321) == 1")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_322() {
    parse_logical_expr_ast("sig322[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_323() {
    parse_logical_expr_ast("sig323 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_324() {
    parse_logical_expr_ast("flag324 ? yes324 : no324")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_325() {
    parse_logical_expr_ast("{2{sig325}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_326() {
    parse_logical_expr_ast("signed'(sig326)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_327() {
    parse_logical_expr_ast("sig327 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_328() {
    parse_logical_expr_ast("~|sig328").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_329() {
    parse_logical_expr_ast("type(state329)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_330() {
    parse_logical_expr_ast("sig330 + 330").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_331() {
    parse_logical_expr_ast("(sig331 & mask331) == 11")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_332() {
    parse_logical_expr_ast("sig332[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_333() {
    parse_logical_expr_ast("sig333 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_334() {
    parse_logical_expr_ast("flag334 ? yes334 : no334")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_335() {
    parse_logical_expr_ast("{2{sig335}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_336() {
    parse_logical_expr_ast("signed'(sig336)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_337() {
    parse_logical_expr_ast("sig337 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_338() {
    parse_logical_expr_ast("~|sig338").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_339() {
    parse_logical_expr_ast("type(state339)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_340() {
    parse_logical_expr_ast("sig340 + 340").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_341() {
    parse_logical_expr_ast("(sig341 & mask341) == 5")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_342() {
    parse_logical_expr_ast("sig342[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_343() {
    parse_logical_expr_ast("sig343 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_344() {
    parse_logical_expr_ast("flag344 ? yes344 : no344")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_345() {
    parse_logical_expr_ast("{2{sig345}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_346() {
    parse_logical_expr_ast("signed'(sig346)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_347() {
    parse_logical_expr_ast("sig347 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_348() {
    parse_logical_expr_ast("~|sig348").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_349() {
    parse_logical_expr_ast("type(state349)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_350() {
    parse_logical_expr_ast("sig350 + 350").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_351() {
    parse_logical_expr_ast("(sig351 & mask351) == 15")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_352() {
    parse_logical_expr_ast("sig352[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_353() {
    parse_logical_expr_ast("sig353 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_354() {
    parse_logical_expr_ast("flag354 ? yes354 : no354")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_355() {
    parse_logical_expr_ast("{2{sig355}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_356() {
    parse_logical_expr_ast("signed'(sig356)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_357() {
    parse_logical_expr_ast("sig357 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_358() {
    parse_logical_expr_ast("~|sig358").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_359() {
    parse_logical_expr_ast("type(state359)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_360() {
    parse_logical_expr_ast("sig360 + 360").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_361() {
    parse_logical_expr_ast("(sig361 & mask361) == 9")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_362() {
    parse_logical_expr_ast("sig362[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_363() {
    parse_logical_expr_ast("sig363 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_364() {
    parse_logical_expr_ast("flag364 ? yes364 : no364")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_365() {
    parse_logical_expr_ast("{2{sig365}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_366() {
    parse_logical_expr_ast("signed'(sig366)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_367() {
    parse_logical_expr_ast("sig367 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_368() {
    parse_logical_expr_ast("~|sig368").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_369() {
    parse_logical_expr_ast("type(state369)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_370() {
    parse_logical_expr_ast("sig370 + 370").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_371() {
    parse_logical_expr_ast("(sig371 & mask371) == 3")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_372() {
    parse_logical_expr_ast("sig372[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_373() {
    parse_logical_expr_ast("sig373 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_374() {
    parse_logical_expr_ast("flag374 ? yes374 : no374")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_375() {
    parse_logical_expr_ast("{2{sig375}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_376() {
    parse_logical_expr_ast("signed'(sig376)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_377() {
    parse_logical_expr_ast("sig377 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_378() {
    parse_logical_expr_ast("~|sig378").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_379() {
    parse_logical_expr_ast("type(state379)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_380() {
    parse_logical_expr_ast("sig380 + 380").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_381() {
    parse_logical_expr_ast("(sig381 & mask381) == 13")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_382() {
    parse_logical_expr_ast("sig382[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_383() {
    parse_logical_expr_ast("sig383 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_384() {
    parse_logical_expr_ast("flag384 ? yes384 : no384")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_385() {
    parse_logical_expr_ast("{2{sig385}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_386() {
    parse_logical_expr_ast("signed'(sig386)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_387() {
    parse_logical_expr_ast("sig387 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_388() {
    parse_logical_expr_ast("~|sig388").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_389() {
    parse_logical_expr_ast("type(state389)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_390() {
    parse_logical_expr_ast("sig390 + 390").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_391() {
    parse_logical_expr_ast("(sig391 & mask391) == 7")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_392() {
    parse_logical_expr_ast("sig392[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_393() {
    parse_logical_expr_ast("sig393 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_394() {
    parse_logical_expr_ast("flag394 ? yes394 : no394")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_395() {
    parse_logical_expr_ast("{2{sig395}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_396() {
    parse_logical_expr_ast("signed'(sig396)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_397() {
    parse_logical_expr_ast("sig397 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_398() {
    parse_logical_expr_ast("~|sig398").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_399() {
    parse_logical_expr_ast("type(state399)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_400() {
    parse_logical_expr_ast("sig400 + 400").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_401() {
    parse_logical_expr_ast("(sig401 & mask401) == 1")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_402() {
    parse_logical_expr_ast("sig402[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_403() {
    parse_logical_expr_ast("sig403 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_404() {
    parse_logical_expr_ast("flag404 ? yes404 : no404")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_405() {
    parse_logical_expr_ast("{2{sig405}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_406() {
    parse_logical_expr_ast("signed'(sig406)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_407() {
    parse_logical_expr_ast("sig407 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_408() {
    parse_logical_expr_ast("~|sig408").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_409() {
    parse_logical_expr_ast("type(state409)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_410() {
    parse_logical_expr_ast("sig410 + 410").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_411() {
    parse_logical_expr_ast("(sig411 & mask411) == 11")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_412() {
    parse_logical_expr_ast("sig412[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_413() {
    parse_logical_expr_ast("sig413 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_414() {
    parse_logical_expr_ast("flag414 ? yes414 : no414")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_415() {
    parse_logical_expr_ast("{2{sig415}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_416() {
    parse_logical_expr_ast("signed'(sig416)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_417() {
    parse_logical_expr_ast("sig417 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_418() {
    parse_logical_expr_ast("~|sig418").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_419() {
    parse_logical_expr_ast("type(state419)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_420() {
    parse_logical_expr_ast("sig420 + 420").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_421() {
    parse_logical_expr_ast("(sig421 & mask421) == 5")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_422() {
    parse_logical_expr_ast("sig422[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_423() {
    parse_logical_expr_ast("sig423 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_424() {
    parse_logical_expr_ast("flag424 ? yes424 : no424")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_425() {
    parse_logical_expr_ast("{2{sig425}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_426() {
    parse_logical_expr_ast("signed'(sig426)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_427() {
    parse_logical_expr_ast("sig427 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_428() {
    parse_logical_expr_ast("~|sig428").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_429() {
    parse_logical_expr_ast("type(state429)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_430() {
    parse_logical_expr_ast("sig430 + 430").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_431() {
    parse_logical_expr_ast("(sig431 & mask431) == 15")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_432() {
    parse_logical_expr_ast("sig432[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_433() {
    parse_logical_expr_ast("sig433 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_434() {
    parse_logical_expr_ast("flag434 ? yes434 : no434")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_435() {
    parse_logical_expr_ast("{2{sig435}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_436() {
    parse_logical_expr_ast("signed'(sig436)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_437() {
    parse_logical_expr_ast("sig437 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_438() {
    parse_logical_expr_ast("~|sig438").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_439() {
    parse_logical_expr_ast("type(state439)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_440() {
    parse_logical_expr_ast("sig440 + 440").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_441() {
    parse_logical_expr_ast("(sig441 & mask441) == 9")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_442() {
    parse_logical_expr_ast("sig442[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_443() {
    parse_logical_expr_ast("sig443 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_444() {
    parse_logical_expr_ast("flag444 ? yes444 : no444")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_445() {
    parse_logical_expr_ast("{2{sig445}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_446() {
    parse_logical_expr_ast("signed'(sig446)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_447() {
    parse_logical_expr_ast("sig447 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_448() {
    parse_logical_expr_ast("~|sig448").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_449() {
    parse_logical_expr_ast("type(state449)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_450() {
    parse_logical_expr_ast("sig450 + 450").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_451() {
    parse_logical_expr_ast("(sig451 & mask451) == 3")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_452() {
    parse_logical_expr_ast("sig452[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_453() {
    parse_logical_expr_ast("sig453 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_454() {
    parse_logical_expr_ast("flag454 ? yes454 : no454")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_455() {
    parse_logical_expr_ast("{2{sig455}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_456() {
    parse_logical_expr_ast("signed'(sig456)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_457() {
    parse_logical_expr_ast("sig457 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_458() {
    parse_logical_expr_ast("~|sig458").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_459() {
    parse_logical_expr_ast("type(state459)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_460() {
    parse_logical_expr_ast("sig460 + 460").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_461() {
    parse_logical_expr_ast("(sig461 & mask461) == 13")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_462() {
    parse_logical_expr_ast("sig462[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_463() {
    parse_logical_expr_ast("sig463 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_464() {
    parse_logical_expr_ast("flag464 ? yes464 : no464")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_465() {
    parse_logical_expr_ast("{2{sig465}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_466() {
    parse_logical_expr_ast("signed'(sig466)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_467() {
    parse_logical_expr_ast("sig467 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_468() {
    parse_logical_expr_ast("~|sig468").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_469() {
    parse_logical_expr_ast("type(state469)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_470() {
    parse_logical_expr_ast("sig470 + 470").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_471() {
    parse_logical_expr_ast("(sig471 & mask471) == 7")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_472() {
    parse_logical_expr_ast("sig472[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_473() {
    parse_logical_expr_ast("sig473 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_474() {
    parse_logical_expr_ast("flag474 ? yes474 : no474")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_475() {
    parse_logical_expr_ast("{2{sig475}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_476() {
    parse_logical_expr_ast("signed'(sig476)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_477() {
    parse_logical_expr_ast("sig477 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_478() {
    parse_logical_expr_ast("~|sig478").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_479() {
    parse_logical_expr_ast("type(state479)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_480() {
    parse_logical_expr_ast("sig480 + 480").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_481() {
    parse_logical_expr_ast("(sig481 & mask481) == 1")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_482() {
    parse_logical_expr_ast("sig482[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_483() {
    parse_logical_expr_ast("sig483 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_484() {
    parse_logical_expr_ast("flag484 ? yes484 : no484")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_485() {
    parse_logical_expr_ast("{2{sig485}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_486() {
    parse_logical_expr_ast("signed'(sig486)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_487() {
    parse_logical_expr_ast("sig487 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_488() {
    parse_logical_expr_ast("~|sig488").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_489() {
    parse_logical_expr_ast("type(state489)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_490() {
    parse_logical_expr_ast("sig490 + 490").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_491() {
    parse_logical_expr_ast("(sig491 & mask491) == 11")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_492() {
    parse_logical_expr_ast("sig492[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_493() {
    parse_logical_expr_ast("sig493 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_494() {
    parse_logical_expr_ast("flag494 ? yes494 : no494")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_495() {
    parse_logical_expr_ast("{2{sig495}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_496() {
    parse_logical_expr_ast("signed'(sig496)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_497() {
    parse_logical_expr_ast("sig497 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_498() {
    parse_logical_expr_ast("~|sig498").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_499() {
    parse_logical_expr_ast("type(state499)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_500() {
    parse_logical_expr_ast("sig500 + 500").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_501() {
    parse_logical_expr_ast("(sig501 & mask501) == 5")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_502() {
    parse_logical_expr_ast("sig502[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_503() {
    parse_logical_expr_ast("sig503 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_504() {
    parse_logical_expr_ast("flag504 ? yes504 : no504")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_505() {
    parse_logical_expr_ast("{2{sig505}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_506() {
    parse_logical_expr_ast("signed'(sig506)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_507() {
    parse_logical_expr_ast("sig507 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_508() {
    parse_logical_expr_ast("~|sig508").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_509() {
    parse_logical_expr_ast("type(state509)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_510() {
    parse_logical_expr_ast("sig510 + 510").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_511() {
    parse_logical_expr_ast("(sig511 & mask511) == 15")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_512() {
    parse_logical_expr_ast("sig512[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_513() {
    parse_logical_expr_ast("sig513 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_514() {
    parse_logical_expr_ast("flag514 ? yes514 : no514")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_515() {
    parse_logical_expr_ast("{2{sig515}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_516() {
    parse_logical_expr_ast("signed'(sig516)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_517() {
    parse_logical_expr_ast("sig517 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_518() {
    parse_logical_expr_ast("~|sig518").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_519() {
    parse_logical_expr_ast("type(state519)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_520() {
    parse_logical_expr_ast("sig520 + 520").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_521() {
    parse_logical_expr_ast("(sig521 & mask521) == 9")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_522() {
    parse_logical_expr_ast("sig522[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_523() {
    parse_logical_expr_ast("sig523 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_524() {
    parse_logical_expr_ast("flag524 ? yes524 : no524")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_525() {
    parse_logical_expr_ast("{2{sig525}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_526() {
    parse_logical_expr_ast("signed'(sig526)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_527() {
    parse_logical_expr_ast("sig527 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_528() {
    parse_logical_expr_ast("~|sig528").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_529() {
    parse_logical_expr_ast("type(state529)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_530() {
    parse_logical_expr_ast("sig530 + 530").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_531() {
    parse_logical_expr_ast("(sig531 & mask531) == 3")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_532() {
    parse_logical_expr_ast("sig532[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_533() {
    parse_logical_expr_ast("sig533 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_534() {
    parse_logical_expr_ast("flag534 ? yes534 : no534")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_535() {
    parse_logical_expr_ast("{2{sig535}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_536() {
    parse_logical_expr_ast("signed'(sig536)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_537() {
    parse_logical_expr_ast("sig537 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_538() {
    parse_logical_expr_ast("~|sig538").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_539() {
    parse_logical_expr_ast("type(state539)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_540() {
    parse_logical_expr_ast("sig540 + 540").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_541() {
    parse_logical_expr_ast("(sig541 & mask541) == 13")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_542() {
    parse_logical_expr_ast("sig542[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_543() {
    parse_logical_expr_ast("sig543 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_544() {
    parse_logical_expr_ast("flag544 ? yes544 : no544")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_545() {
    parse_logical_expr_ast("{2{sig545}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_546() {
    parse_logical_expr_ast("signed'(sig546)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_547() {
    parse_logical_expr_ast("sig547 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_548() {
    parse_logical_expr_ast("~|sig548").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_549() {
    parse_logical_expr_ast("type(state549)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_550() {
    parse_logical_expr_ast("sig550 + 550").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_551() {
    parse_logical_expr_ast("(sig551 & mask551) == 7")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_552() {
    parse_logical_expr_ast("sig552[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_553() {
    parse_logical_expr_ast("sig553 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_554() {
    parse_logical_expr_ast("flag554 ? yes554 : no554")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_555() {
    parse_logical_expr_ast("{2{sig555}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_556() {
    parse_logical_expr_ast("signed'(sig556)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_557() {
    parse_logical_expr_ast("sig557 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_558() {
    parse_logical_expr_ast("~|sig558").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_559() {
    parse_logical_expr_ast("type(state559)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_560() {
    parse_logical_expr_ast("sig560 + 560").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_561() {
    parse_logical_expr_ast("(sig561 & mask561) == 1")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_562() {
    parse_logical_expr_ast("sig562[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_563() {
    parse_logical_expr_ast("sig563 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_564() {
    parse_logical_expr_ast("flag564 ? yes564 : no564")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_565() {
    parse_logical_expr_ast("{2{sig565}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_566() {
    parse_logical_expr_ast("signed'(sig566)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_567() {
    parse_logical_expr_ast("sig567 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_568() {
    parse_logical_expr_ast("~|sig568").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_569() {
    parse_logical_expr_ast("type(state569)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_570() {
    parse_logical_expr_ast("sig570 + 570").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_571() {
    parse_logical_expr_ast("(sig571 & mask571) == 11")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_572() {
    parse_logical_expr_ast("sig572[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_573() {
    parse_logical_expr_ast("sig573 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_574() {
    parse_logical_expr_ast("flag574 ? yes574 : no574")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_575() {
    parse_logical_expr_ast("{2{sig575}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_576() {
    parse_logical_expr_ast("signed'(sig576)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_577() {
    parse_logical_expr_ast("sig577 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_578() {
    parse_logical_expr_ast("~|sig578").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_579() {
    parse_logical_expr_ast("type(state579)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_580() {
    parse_logical_expr_ast("sig580 + 580").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_581() {
    parse_logical_expr_ast("(sig581 & mask581) == 5")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_582() {
    parse_logical_expr_ast("sig582[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_583() {
    parse_logical_expr_ast("sig583 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_584() {
    parse_logical_expr_ast("flag584 ? yes584 : no584")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_585() {
    parse_logical_expr_ast("{2{sig585}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_586() {
    parse_logical_expr_ast("signed'(sig586)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_587() {
    parse_logical_expr_ast("sig587 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_588() {
    parse_logical_expr_ast("~|sig588").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_589() {
    parse_logical_expr_ast("type(state589)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_590() {
    parse_logical_expr_ast("sig590 + 590").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_591() {
    parse_logical_expr_ast("(sig591 & mask591) == 15")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_592() {
    parse_logical_expr_ast("sig592[0] == 1'b1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_593() {
    parse_logical_expr_ast("sig593 inside {1, 2, 3}")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_594() {
    parse_logical_expr_ast("flag594 ? yes594 : no594")
        .expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_595() {
    parse_logical_expr_ast("{2{sig595}}").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_596() {
    parse_logical_expr_ast("signed'(sig596)").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_597() {
    parse_logical_expr_ast("sig597 << 1").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_598() {
    parse_logical_expr_ast("~|sig598").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_599() {
    parse_logical_expr_ast("type(state599)::IDLE").expect("logical smoke expression should parse");
}

#[test]
fn parser_smoke_logical_600() {
    parse_logical_expr_ast("sig600 + 600").expect("logical smoke expression should parse");
}
