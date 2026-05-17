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
