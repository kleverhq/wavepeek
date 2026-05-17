use super::{parse_event_expr_ast, parse_logical_expr_ast};

#[test]
fn parser_surface_event_aaa() {
    parse_event_expr_ast("posedge clkaaa").expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aab() {
    parse_event_expr_ast("negedge rstaab iff enableaab")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aac() {
    parse_event_expr_ast("eventaac or posedge clkaac")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aad() {
    parse_event_expr_ast("aaad, baad, caad").expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aae() {
    parse_event_expr_ast("edge dataaae iff validaae")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aaf() {
    parse_event_expr_ast("posedge clkaaf").expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aag() {
    parse_event_expr_ast("negedge rstaag iff enableaag")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aah() {
    parse_event_expr_ast("eventaah or posedge clkaah")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aai() {
    parse_event_expr_ast("aaai, baai, caai").expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aaj() {
    parse_event_expr_ast("edge dataaaj iff validaaj")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aak() {
    parse_event_expr_ast("posedge clkaak").expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aal() {
    parse_event_expr_ast("negedge rstaal iff enableaal")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aam() {
    parse_event_expr_ast("eventaam or posedge clkaam")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aan() {
    parse_event_expr_ast("aaan, baan, caan").expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aao() {
    parse_event_expr_ast("edge dataaao iff validaao")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aap() {
    parse_event_expr_ast("posedge clkaap").expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aaq() {
    parse_event_expr_ast("negedge rstaaq iff enableaaq")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aar() {
    parse_event_expr_ast("eventaar or posedge clkaar")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aas() {
    parse_event_expr_ast("aaas, baas, caas").expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aat() {
    parse_event_expr_ast("edge dataaat iff validaat")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aau() {
    parse_event_expr_ast("posedge clkaau").expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aav() {
    parse_event_expr_ast("negedge rstaav iff enableaav")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aaw() {
    parse_event_expr_ast("eventaaw or posedge clkaaw")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aax() {
    parse_event_expr_ast("aaax, baax, caax").expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aay() {
    parse_event_expr_ast("edge dataaay iff validaay")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aaz() {
    parse_event_expr_ast("posedge clkaaz").expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_aba() {
    parse_event_expr_ast("negedge rstaba iff enableaba")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_abb() {
    parse_event_expr_ast("eventabb or posedge clkabb")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_abc() {
    parse_event_expr_ast("aabc, babc, cabc").expect("event surface expression should parse");
}

#[test]
fn parser_surface_event_abd() {
    parse_event_expr_ast("edge dataabd iff validabd")
        .expect("event surface expression should parse");
}

#[test]
fn parser_surface_logical_aaa() {
    parse_logical_expr_ast("sigaaa + 0").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aab() {
    parse_logical_expr_ast("(sigaab & maskaab) == 1")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aac() {
    parse_logical_expr_ast("sigaac[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aad() {
    parse_logical_expr_ast("sigaad[3:0] != 4'h3").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aae() {
    parse_logical_expr_ast("sigaae inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aaf() {
    parse_logical_expr_ast("flagaaf ? yesaaf : noaaf")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aag() {
    parse_logical_expr_ast("{2{sigaag}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aah() {
    parse_logical_expr_ast("sigaah << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aai() {
    parse_logical_expr_ast("sigaai >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aaj() {
    parse_logical_expr_ast("~&sigaaj").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aak() {
    parse_logical_expr_ast("^~sigaak").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aal() {
    parse_logical_expr_ast("type(stateaal)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aam() {
    parse_logical_expr_ast("logic[8]'(sigaam)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aan() {
    parse_logical_expr_ast("unsigned'(sigaan)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aao() {
    parse_logical_expr_ast("sigaao.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aap() {
    parse_logical_expr_ast("{sigaap, maskaap} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aaq() {
    parse_logical_expr_ast("sigaaq ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aar() {
    parse_logical_expr_ast("signed'(sigaar)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aas() {
    parse_logical_expr_ast("sigaas >= thresholdaas")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aat() {
    parse_logical_expr_ast("sigaat || readyaat").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aau() {
    parse_logical_expr_ast("sigaau + 3").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aav() {
    parse_logical_expr_ast("(sigaav & maskaav) == 5")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aaw() {
    parse_logical_expr_ast("sigaaw[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aax() {
    parse_logical_expr_ast("sigaax[3:0] != 4'h7").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aay() {
    parse_logical_expr_ast("sigaay inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aaz() {
    parse_logical_expr_ast("flagaaz ? yesaaz : noaaz")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aba() {
    parse_logical_expr_ast("{2{sigaba}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abb() {
    parse_logical_expr_ast("sigabb << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abc() {
    parse_logical_expr_ast("sigabc >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abd() {
    parse_logical_expr_ast("~&sigabd").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abe() {
    parse_logical_expr_ast("^~sigabe").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abf() {
    parse_logical_expr_ast("type(stateabf)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abg() {
    parse_logical_expr_ast("logic[8]'(sigabg)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abh() {
    parse_logical_expr_ast("unsigned'(sigabh)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abi() {
    parse_logical_expr_ast("sigabi.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abj() {
    parse_logical_expr_ast("{sigabj, maskabj} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abk() {
    parse_logical_expr_ast("sigabk ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abl() {
    parse_logical_expr_ast("signed'(sigabl)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abm() {
    parse_logical_expr_ast("sigabm >= thresholdabm")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abn() {
    parse_logical_expr_ast("sigabn || readyabn").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abo() {
    parse_logical_expr_ast("sigabo + 6").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abp() {
    parse_logical_expr_ast("(sigabp & maskabp) == 9")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abq() {
    parse_logical_expr_ast("sigabq[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abr() {
    parse_logical_expr_ast("sigabr[3:0] != 4'hb").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abs() {
    parse_logical_expr_ast("sigabs inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abt() {
    parse_logical_expr_ast("flagabt ? yesabt : noabt")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abu() {
    parse_logical_expr_ast("{2{sigabu}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abv() {
    parse_logical_expr_ast("sigabv << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abw() {
    parse_logical_expr_ast("sigabw >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abx() {
    parse_logical_expr_ast("~&sigabx").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aby() {
    parse_logical_expr_ast("^~sigaby").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_abz() {
    parse_logical_expr_ast("type(stateabz)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aca() {
    parse_logical_expr_ast("logic[8]'(sigaca)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acb() {
    parse_logical_expr_ast("unsigned'(sigacb)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acc() {
    parse_logical_expr_ast("sigacc.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acd() {
    parse_logical_expr_ast("{sigacd, maskacd} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ace() {
    parse_logical_expr_ast("sigace ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acf() {
    parse_logical_expr_ast("signed'(sigacf)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acg() {
    parse_logical_expr_ast("sigacg >= thresholdacg")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ach() {
    parse_logical_expr_ast("sigach || readyach").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aci() {
    parse_logical_expr_ast("sigaci + 9").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acj() {
    parse_logical_expr_ast("(sigacj & maskacj) == 13")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ack() {
    parse_logical_expr_ast("sigack[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acl() {
    parse_logical_expr_ast("sigacl[3:0] != 4'hf").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acm() {
    parse_logical_expr_ast("sigacm inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acn() {
    parse_logical_expr_ast("flagacn ? yesacn : noacn")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aco() {
    parse_logical_expr_ast("{2{sigaco}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acp() {
    parse_logical_expr_ast("sigacp << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acq() {
    parse_logical_expr_ast("sigacq >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acr() {
    parse_logical_expr_ast("~&sigacr").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acs() {
    parse_logical_expr_ast("^~sigacs").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_act() {
    parse_logical_expr_ast("type(stateact)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acu() {
    parse_logical_expr_ast("logic[8]'(sigacu)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acv() {
    parse_logical_expr_ast("unsigned'(sigacv)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acw() {
    parse_logical_expr_ast("sigacw.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acx() {
    parse_logical_expr_ast("{sigacx, maskacx} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acy() {
    parse_logical_expr_ast("sigacy ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_acz() {
    parse_logical_expr_ast("signed'(sigacz)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ada() {
    parse_logical_expr_ast("sigada >= thresholdada")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adb() {
    parse_logical_expr_ast("sigadb || readyadb").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adc() {
    parse_logical_expr_ast("sigadc + 12").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_add() {
    parse_logical_expr_ast("(sigadd & maskadd) == 1")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ade() {
    parse_logical_expr_ast("sigade[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adf() {
    parse_logical_expr_ast("sigadf[3:0] != 4'h3").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adg() {
    parse_logical_expr_ast("sigadg inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adh() {
    parse_logical_expr_ast("flagadh ? yesadh : noadh")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adi() {
    parse_logical_expr_ast("{2{sigadi}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adj() {
    parse_logical_expr_ast("sigadj << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adk() {
    parse_logical_expr_ast("sigadk >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adl() {
    parse_logical_expr_ast("~&sigadl").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adm() {
    parse_logical_expr_ast("^~sigadm").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adn() {
    parse_logical_expr_ast("type(stateadn)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ado() {
    parse_logical_expr_ast("logic[8]'(sigado)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adp() {
    parse_logical_expr_ast("unsigned'(sigadp)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adq() {
    parse_logical_expr_ast("sigadq.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adr() {
    parse_logical_expr_ast("{sigadr, maskadr} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ads() {
    parse_logical_expr_ast("sigads ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adt() {
    parse_logical_expr_ast("signed'(sigadt)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adu() {
    parse_logical_expr_ast("sigadu >= thresholdadu")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adv() {
    parse_logical_expr_ast("sigadv || readyadv").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adw() {
    parse_logical_expr_ast("sigadw + 15").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adx() {
    parse_logical_expr_ast("(sigadx & maskadx) == 5")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ady() {
    parse_logical_expr_ast("sigady[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_adz() {
    parse_logical_expr_ast("sigadz[3:0] != 4'h7").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aea() {
    parse_logical_expr_ast("sigaea inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aeb() {
    parse_logical_expr_ast("flagaeb ? yesaeb : noaeb")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aec() {
    parse_logical_expr_ast("{2{sigaec}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aed() {
    parse_logical_expr_ast("sigaed << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aee() {
    parse_logical_expr_ast("sigaee >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aef() {
    parse_logical_expr_ast("~&sigaef").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aeg() {
    parse_logical_expr_ast("^~sigaeg").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aeh() {
    parse_logical_expr_ast("type(stateaeh)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aei() {
    parse_logical_expr_ast("logic[8]'(sigaei)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aej() {
    parse_logical_expr_ast("unsigned'(sigaej)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aek() {
    parse_logical_expr_ast("sigaek.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ael() {
    parse_logical_expr_ast("{sigael, maskael} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aem() {
    parse_logical_expr_ast("sigaem ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aen() {
    parse_logical_expr_ast("signed'(sigaen)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aeo() {
    parse_logical_expr_ast("sigaeo >= thresholdaeo")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aep() {
    parse_logical_expr_ast("sigaep || readyaep").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aeq() {
    parse_logical_expr_ast("sigaeq + 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aer() {
    parse_logical_expr_ast("(sigaer & maskaer) == 9")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aes() {
    parse_logical_expr_ast("sigaes[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aet() {
    parse_logical_expr_ast("sigaet[3:0] != 4'hb").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aeu() {
    parse_logical_expr_ast("sigaeu inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aev() {
    parse_logical_expr_ast("flagaev ? yesaev : noaev")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aew() {
    parse_logical_expr_ast("{2{sigaew}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aex() {
    parse_logical_expr_ast("sigaex << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aey() {
    parse_logical_expr_ast("sigaey >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aez() {
    parse_logical_expr_ast("~&sigaez").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afa() {
    parse_logical_expr_ast("^~sigafa").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afb() {
    parse_logical_expr_ast("type(stateafb)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afc() {
    parse_logical_expr_ast("logic[8]'(sigafc)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afd() {
    parse_logical_expr_ast("unsigned'(sigafd)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afe() {
    parse_logical_expr_ast("sigafe.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aff() {
    parse_logical_expr_ast("{sigaff, maskaff} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afg() {
    parse_logical_expr_ast("sigafg ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afh() {
    parse_logical_expr_ast("signed'(sigafh)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afi() {
    parse_logical_expr_ast("sigafi >= thresholdafi")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afj() {
    parse_logical_expr_ast("sigafj || readyafj").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afk() {
    parse_logical_expr_ast("sigafk + 4").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afl() {
    parse_logical_expr_ast("(sigafl & maskafl) == 13")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afm() {
    parse_logical_expr_ast("sigafm[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afn() {
    parse_logical_expr_ast("sigafn[3:0] != 4'hf").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afo() {
    parse_logical_expr_ast("sigafo inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afp() {
    parse_logical_expr_ast("flagafp ? yesafp : noafp")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afq() {
    parse_logical_expr_ast("{2{sigafq}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afr() {
    parse_logical_expr_ast("sigafr << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afs() {
    parse_logical_expr_ast("sigafs >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aft() {
    parse_logical_expr_ast("~&sigaft").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afu() {
    parse_logical_expr_ast("^~sigafu").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afv() {
    parse_logical_expr_ast("type(stateafv)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afw() {
    parse_logical_expr_ast("logic[8]'(sigafw)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afx() {
    parse_logical_expr_ast("unsigned'(sigafx)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afy() {
    parse_logical_expr_ast("sigafy.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_afz() {
    parse_logical_expr_ast("{sigafz, maskafz} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aga() {
    parse_logical_expr_ast("sigaga ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agb() {
    parse_logical_expr_ast("signed'(sigagb)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agc() {
    parse_logical_expr_ast("sigagc >= thresholdagc")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agd() {
    parse_logical_expr_ast("sigagd || readyagd").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_age() {
    parse_logical_expr_ast("sigage + 7").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agf() {
    parse_logical_expr_ast("(sigagf & maskagf) == 1")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agg() {
    parse_logical_expr_ast("sigagg[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agh() {
    parse_logical_expr_ast("sigagh[3:0] != 4'h3").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agi() {
    parse_logical_expr_ast("sigagi inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agj() {
    parse_logical_expr_ast("flagagj ? yesagj : noagj")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agk() {
    parse_logical_expr_ast("{2{sigagk}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agl() {
    parse_logical_expr_ast("sigagl << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agm() {
    parse_logical_expr_ast("sigagm >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agn() {
    parse_logical_expr_ast("~&sigagn").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ago() {
    parse_logical_expr_ast("^~sigago").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agp() {
    parse_logical_expr_ast("type(stateagp)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agq() {
    parse_logical_expr_ast("logic[8]'(sigagq)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agr() {
    parse_logical_expr_ast("unsigned'(sigagr)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ags() {
    parse_logical_expr_ast("sigags.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agt() {
    parse_logical_expr_ast("{sigagt, maskagt} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agu() {
    parse_logical_expr_ast("sigagu ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agv() {
    parse_logical_expr_ast("signed'(sigagv)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agw() {
    parse_logical_expr_ast("sigagw >= thresholdagw")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agx() {
    parse_logical_expr_ast("sigagx || readyagx").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agy() {
    parse_logical_expr_ast("sigagy + 10").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_agz() {
    parse_logical_expr_ast("(sigagz & maskagz) == 5")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aha() {
    parse_logical_expr_ast("sigaha[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahb() {
    parse_logical_expr_ast("sigahb[3:0] != 4'h7").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahc() {
    parse_logical_expr_ast("sigahc inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahd() {
    parse_logical_expr_ast("flagahd ? yesahd : noahd")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahe() {
    parse_logical_expr_ast("{2{sigahe}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahf() {
    parse_logical_expr_ast("sigahf << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahg() {
    parse_logical_expr_ast("sigahg >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahh() {
    parse_logical_expr_ast("~&sigahh").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahi() {
    parse_logical_expr_ast("^~sigahi").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahj() {
    parse_logical_expr_ast("type(stateahj)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahk() {
    parse_logical_expr_ast("logic[8]'(sigahk)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahl() {
    parse_logical_expr_ast("unsigned'(sigahl)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahm() {
    parse_logical_expr_ast("sigahm.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahn() {
    parse_logical_expr_ast("{sigahn, maskahn} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aho() {
    parse_logical_expr_ast("sigaho ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahp() {
    parse_logical_expr_ast("signed'(sigahp)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahq() {
    parse_logical_expr_ast("sigahq >= thresholdahq")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahr() {
    parse_logical_expr_ast("sigahr || readyahr").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahs() {
    parse_logical_expr_ast("sigahs + 13").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aht() {
    parse_logical_expr_ast("(sigaht & maskaht) == 9")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahu() {
    parse_logical_expr_ast("sigahu[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahv() {
    parse_logical_expr_ast("sigahv[3:0] != 4'hb").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahw() {
    parse_logical_expr_ast("sigahw inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahx() {
    parse_logical_expr_ast("flagahx ? yesahx : noahx")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahy() {
    parse_logical_expr_ast("{2{sigahy}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ahz() {
    parse_logical_expr_ast("sigahz << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aia() {
    parse_logical_expr_ast("sigaia >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aib() {
    parse_logical_expr_ast("~&sigaib").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aic() {
    parse_logical_expr_ast("^~sigaic").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aid() {
    parse_logical_expr_ast("type(stateaid)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aie() {
    parse_logical_expr_ast("logic[8]'(sigaie)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aif() {
    parse_logical_expr_ast("unsigned'(sigaif)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aig() {
    parse_logical_expr_ast("sigaig.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aih() {
    parse_logical_expr_ast("{sigaih, maskaih} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aii() {
    parse_logical_expr_ast("sigaii ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aij() {
    parse_logical_expr_ast("signed'(sigaij)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aik() {
    parse_logical_expr_ast("sigaik >= thresholdaik")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ail() {
    parse_logical_expr_ast("sigail || readyail").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aim() {
    parse_logical_expr_ast("sigaim + 16").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ain() {
    parse_logical_expr_ast("(sigain & maskain) == 13")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aio() {
    parse_logical_expr_ast("sigaio[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aip() {
    parse_logical_expr_ast("sigaip[3:0] != 4'hf").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aiq() {
    parse_logical_expr_ast("sigaiq inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_air() {
    parse_logical_expr_ast("flagair ? yesair : noair")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ais() {
    parse_logical_expr_ast("{2{sigais}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ait() {
    parse_logical_expr_ast("sigait << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aiu() {
    parse_logical_expr_ast("sigaiu >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aiv() {
    parse_logical_expr_ast("~&sigaiv").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aiw() {
    parse_logical_expr_ast("^~sigaiw").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aix() {
    parse_logical_expr_ast("type(stateaix)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aiy() {
    parse_logical_expr_ast("logic[8]'(sigaiy)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aiz() {
    parse_logical_expr_ast("unsigned'(sigaiz)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aja() {
    parse_logical_expr_ast("sigaja.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajb() {
    parse_logical_expr_ast("{sigajb, maskajb} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajc() {
    parse_logical_expr_ast("sigajc ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajd() {
    parse_logical_expr_ast("signed'(sigajd)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aje() {
    parse_logical_expr_ast("sigaje >= thresholdaje")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajf() {
    parse_logical_expr_ast("sigajf || readyajf").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajg() {
    parse_logical_expr_ast("sigajg + 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajh() {
    parse_logical_expr_ast("(sigajh & maskajh) == 1")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aji() {
    parse_logical_expr_ast("sigaji[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajj() {
    parse_logical_expr_ast("sigajj[3:0] != 4'h3").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajk() {
    parse_logical_expr_ast("sigajk inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajl() {
    parse_logical_expr_ast("flagajl ? yesajl : noajl")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajm() {
    parse_logical_expr_ast("{2{sigajm}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajn() {
    parse_logical_expr_ast("sigajn << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajo() {
    parse_logical_expr_ast("sigajo >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajp() {
    parse_logical_expr_ast("~&sigajp").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajq() {
    parse_logical_expr_ast("^~sigajq").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajr() {
    parse_logical_expr_ast("type(stateajr)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajs() {
    parse_logical_expr_ast("logic[8]'(sigajs)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajt() {
    parse_logical_expr_ast("unsigned'(sigajt)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aju() {
    parse_logical_expr_ast("sigaju.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajv() {
    parse_logical_expr_ast("{sigajv, maskajv} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajw() {
    parse_logical_expr_ast("sigajw ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajx() {
    parse_logical_expr_ast("signed'(sigajx)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajy() {
    parse_logical_expr_ast("sigajy >= thresholdajy")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ajz() {
    parse_logical_expr_ast("sigajz || readyajz").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aka() {
    parse_logical_expr_ast("sigaka + 5").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akb() {
    parse_logical_expr_ast("(sigakb & maskakb) == 5")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akc() {
    parse_logical_expr_ast("sigakc[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akd() {
    parse_logical_expr_ast("sigakd[3:0] != 4'h7").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ake() {
    parse_logical_expr_ast("sigake inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akf() {
    parse_logical_expr_ast("flagakf ? yesakf : noakf")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akg() {
    parse_logical_expr_ast("{2{sigakg}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akh() {
    parse_logical_expr_ast("sigakh << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aki() {
    parse_logical_expr_ast("sigaki >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akj() {
    parse_logical_expr_ast("~&sigakj").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akk() {
    parse_logical_expr_ast("^~sigakk").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akl() {
    parse_logical_expr_ast("type(stateakl)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akm() {
    parse_logical_expr_ast("logic[8]'(sigakm)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akn() {
    parse_logical_expr_ast("unsigned'(sigakn)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ako() {
    parse_logical_expr_ast("sigako.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akp() {
    parse_logical_expr_ast("{sigakp, maskakp} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akq() {
    parse_logical_expr_ast("sigakq ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akr() {
    parse_logical_expr_ast("signed'(sigakr)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aks() {
    parse_logical_expr_ast("sigaks >= thresholdaks")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akt() {
    parse_logical_expr_ast("sigakt || readyakt").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aku() {
    parse_logical_expr_ast("sigaku + 8").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akv() {
    parse_logical_expr_ast("(sigakv & maskakv) == 9")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akw() {
    parse_logical_expr_ast("sigakw[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akx() {
    parse_logical_expr_ast("sigakx[3:0] != 4'hb").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aky() {
    parse_logical_expr_ast("sigaky inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_akz() {
    parse_logical_expr_ast("flagakz ? yesakz : noakz")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ala() {
    parse_logical_expr_ast("{2{sigala}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alb() {
    parse_logical_expr_ast("sigalb << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alc() {
    parse_logical_expr_ast("sigalc >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ald() {
    parse_logical_expr_ast("~&sigald").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ale() {
    parse_logical_expr_ast("^~sigale").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alf() {
    parse_logical_expr_ast("type(statealf)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alg() {
    parse_logical_expr_ast("logic[8]'(sigalg)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alh() {
    parse_logical_expr_ast("unsigned'(sigalh)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ali() {
    parse_logical_expr_ast("sigali.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alj() {
    parse_logical_expr_ast("{sigalj, maskalj} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alk() {
    parse_logical_expr_ast("sigalk ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_all() {
    parse_logical_expr_ast("signed'(sigall)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alm() {
    parse_logical_expr_ast("sigalm >= thresholdalm")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aln() {
    parse_logical_expr_ast("sigaln || readyaln").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alo() {
    parse_logical_expr_ast("sigalo + 11").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alp() {
    parse_logical_expr_ast("(sigalp & maskalp) == 13")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alq() {
    parse_logical_expr_ast("sigalq[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alr() {
    parse_logical_expr_ast("sigalr[3:0] != 4'hf").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_als() {
    parse_logical_expr_ast("sigals inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alt() {
    parse_logical_expr_ast("flagalt ? yesalt : noalt")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alu() {
    parse_logical_expr_ast("{2{sigalu}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alv() {
    parse_logical_expr_ast("sigalv << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alw() {
    parse_logical_expr_ast("sigalw >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alx() {
    parse_logical_expr_ast("~&sigalx").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aly() {
    parse_logical_expr_ast("^~sigaly").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_alz() {
    parse_logical_expr_ast("type(statealz)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ama() {
    parse_logical_expr_ast("logic[8]'(sigama)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amb() {
    parse_logical_expr_ast("unsigned'(sigamb)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amc() {
    parse_logical_expr_ast("sigamc.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amd() {
    parse_logical_expr_ast("{sigamd, maskamd} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ame() {
    parse_logical_expr_ast("sigame ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amf() {
    parse_logical_expr_ast("signed'(sigamf)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amg() {
    parse_logical_expr_ast("sigamg >= thresholdamg")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amh() {
    parse_logical_expr_ast("sigamh || readyamh").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ami() {
    parse_logical_expr_ast("sigami + 14").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amj() {
    parse_logical_expr_ast("(sigamj & maskamj) == 1")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amk() {
    parse_logical_expr_ast("sigamk[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aml() {
    parse_logical_expr_ast("sigaml[3:0] != 4'h3").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amm() {
    parse_logical_expr_ast("sigamm inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amn() {
    parse_logical_expr_ast("flagamn ? yesamn : noamn")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amo() {
    parse_logical_expr_ast("{2{sigamo}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amp() {
    parse_logical_expr_ast("sigamp << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amq() {
    parse_logical_expr_ast("sigamq >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amr() {
    parse_logical_expr_ast("~&sigamr").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ams() {
    parse_logical_expr_ast("^~sigams").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amt() {
    parse_logical_expr_ast("type(stateamt)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amu() {
    parse_logical_expr_ast("logic[8]'(sigamu)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amv() {
    parse_logical_expr_ast("unsigned'(sigamv)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amw() {
    parse_logical_expr_ast("sigamw.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amx() {
    parse_logical_expr_ast("{sigamx, maskamx} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amy() {
    parse_logical_expr_ast("sigamy ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_amz() {
    parse_logical_expr_ast("signed'(sigamz)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ana() {
    parse_logical_expr_ast("sigana >= thresholdana")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anb() {
    parse_logical_expr_ast("siganb || readyanb").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anc() {
    parse_logical_expr_ast("siganc + 0").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_and() {
    parse_logical_expr_ast("(sigand & maskand) == 5")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ane() {
    parse_logical_expr_ast("sigane[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anf() {
    parse_logical_expr_ast("siganf[3:0] != 4'h7").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ang() {
    parse_logical_expr_ast("sigang inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anh() {
    parse_logical_expr_ast("flaganh ? yesanh : noanh")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ani() {
    parse_logical_expr_ast("{2{sigani}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anj() {
    parse_logical_expr_ast("siganj << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ank() {
    parse_logical_expr_ast("sigank >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anl() {
    parse_logical_expr_ast("~&siganl").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anm() {
    parse_logical_expr_ast("^~siganm").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ann() {
    parse_logical_expr_ast("type(stateann)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ano() {
    parse_logical_expr_ast("logic[8]'(sigano)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anp() {
    parse_logical_expr_ast("unsigned'(siganp)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anq() {
    parse_logical_expr_ast("siganq.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anr() {
    parse_logical_expr_ast("{siganr, maskanr} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ans() {
    parse_logical_expr_ast("sigans ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ant() {
    parse_logical_expr_ast("signed'(sigant)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anu() {
    parse_logical_expr_ast("siganu >= thresholdanu")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anv() {
    parse_logical_expr_ast("siganv || readyanv").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anw() {
    parse_logical_expr_ast("siganw + 3").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anx() {
    parse_logical_expr_ast("(siganx & maskanx) == 9")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_any() {
    parse_logical_expr_ast("sigany[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_anz() {
    parse_logical_expr_ast("siganz[3:0] != 4'hb").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aoa() {
    parse_logical_expr_ast("sigaoa inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aob() {
    parse_logical_expr_ast("flagaob ? yesaob : noaob")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aoc() {
    parse_logical_expr_ast("{2{sigaoc}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aod() {
    parse_logical_expr_ast("sigaod << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aoe() {
    parse_logical_expr_ast("sigaoe >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aof() {
    parse_logical_expr_ast("~&sigaof").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aog() {
    parse_logical_expr_ast("^~sigaog").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aoh() {
    parse_logical_expr_ast("type(stateaoh)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aoi() {
    parse_logical_expr_ast("logic[8]'(sigaoi)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aoj() {
    parse_logical_expr_ast("unsigned'(sigaoj)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aok() {
    parse_logical_expr_ast("sigaok.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aol() {
    parse_logical_expr_ast("{sigaol, maskaol} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aom() {
    parse_logical_expr_ast("sigaom ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aon() {
    parse_logical_expr_ast("signed'(sigaon)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aoo() {
    parse_logical_expr_ast("sigaoo >= thresholdaoo")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aop() {
    parse_logical_expr_ast("sigaop || readyaop").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aoq() {
    parse_logical_expr_ast("sigaoq + 6").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aor() {
    parse_logical_expr_ast("(sigaor & maskaor) == 13")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aos() {
    parse_logical_expr_ast("sigaos[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aot() {
    parse_logical_expr_ast("sigaot[3:0] != 4'hf").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aou() {
    parse_logical_expr_ast("sigaou inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aov() {
    parse_logical_expr_ast("flagaov ? yesaov : noaov")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aow() {
    parse_logical_expr_ast("{2{sigaow}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aox() {
    parse_logical_expr_ast("sigaox << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aoy() {
    parse_logical_expr_ast("sigaoy >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aoz() {
    parse_logical_expr_ast("~&sigaoz").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apa() {
    parse_logical_expr_ast("^~sigapa").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apb() {
    parse_logical_expr_ast("type(stateapb)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apc() {
    parse_logical_expr_ast("logic[8]'(sigapc)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apd() {
    parse_logical_expr_ast("unsigned'(sigapd)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ape() {
    parse_logical_expr_ast("sigape.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apf() {
    parse_logical_expr_ast("{sigapf, maskapf} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apg() {
    parse_logical_expr_ast("sigapg ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aph() {
    parse_logical_expr_ast("signed'(sigaph)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_api() {
    parse_logical_expr_ast("sigapi >= thresholdapi")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apj() {
    parse_logical_expr_ast("sigapj || readyapj").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apk() {
    parse_logical_expr_ast("sigapk + 9").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apl() {
    parse_logical_expr_ast("(sigapl & maskapl) == 1")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apm() {
    parse_logical_expr_ast("sigapm[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apn() {
    parse_logical_expr_ast("sigapn[3:0] != 4'h3").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apo() {
    parse_logical_expr_ast("sigapo inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_app() {
    parse_logical_expr_ast("flagapp ? yesapp : noapp")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apq() {
    parse_logical_expr_ast("{2{sigapq}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apr() {
    parse_logical_expr_ast("sigapr << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aps() {
    parse_logical_expr_ast("sigaps >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apt() {
    parse_logical_expr_ast("~&sigapt").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apu() {
    parse_logical_expr_ast("^~sigapu").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apv() {
    parse_logical_expr_ast("type(stateapv)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apw() {
    parse_logical_expr_ast("logic[8]'(sigapw)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apx() {
    parse_logical_expr_ast("unsigned'(sigapx)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apy() {
    parse_logical_expr_ast("sigapy.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_apz() {
    parse_logical_expr_ast("{sigapz, maskapz} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqa() {
    parse_logical_expr_ast("sigaqa ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqb() {
    parse_logical_expr_ast("signed'(sigaqb)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqc() {
    parse_logical_expr_ast("sigaqc >= thresholdaqc")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqd() {
    parse_logical_expr_ast("sigaqd || readyaqd").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqe() {
    parse_logical_expr_ast("sigaqe + 12").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqf() {
    parse_logical_expr_ast("(sigaqf & maskaqf) == 5")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqg() {
    parse_logical_expr_ast("sigaqg[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqh() {
    parse_logical_expr_ast("sigaqh[3:0] != 4'h7").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqi() {
    parse_logical_expr_ast("sigaqi inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqj() {
    parse_logical_expr_ast("flagaqj ? yesaqj : noaqj")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqk() {
    parse_logical_expr_ast("{2{sigaqk}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aql() {
    parse_logical_expr_ast("sigaql << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqm() {
    parse_logical_expr_ast("sigaqm >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqn() {
    parse_logical_expr_ast("~&sigaqn").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqo() {
    parse_logical_expr_ast("^~sigaqo").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqp() {
    parse_logical_expr_ast("type(stateaqp)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqq() {
    parse_logical_expr_ast("logic[8]'(sigaqq)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqr() {
    parse_logical_expr_ast("unsigned'(sigaqr)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqs() {
    parse_logical_expr_ast("sigaqs.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqt() {
    parse_logical_expr_ast("{sigaqt, maskaqt} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqu() {
    parse_logical_expr_ast("sigaqu ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqv() {
    parse_logical_expr_ast("signed'(sigaqv)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqw() {
    parse_logical_expr_ast("sigaqw >= thresholdaqw")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqx() {
    parse_logical_expr_ast("sigaqx || readyaqx").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqy() {
    parse_logical_expr_ast("sigaqy + 15").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aqz() {
    parse_logical_expr_ast("(sigaqz & maskaqz) == 9")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ara() {
    parse_logical_expr_ast("sigara[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arb() {
    parse_logical_expr_ast("sigarb[3:0] != 4'hb").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arc() {
    parse_logical_expr_ast("sigarc inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ard() {
    parse_logical_expr_ast("flagard ? yesard : noard")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_are() {
    parse_logical_expr_ast("{2{sigare}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arf() {
    parse_logical_expr_ast("sigarf << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arg() {
    parse_logical_expr_ast("sigarg >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arh() {
    parse_logical_expr_ast("~&sigarh").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ari() {
    parse_logical_expr_ast("^~sigari").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arj() {
    parse_logical_expr_ast("type(statearj)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ark() {
    parse_logical_expr_ast("logic[8]'(sigark)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arl() {
    parse_logical_expr_ast("unsigned'(sigarl)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arm() {
    parse_logical_expr_ast("sigarm.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arn() {
    parse_logical_expr_ast("{sigarn, maskarn} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aro() {
    parse_logical_expr_ast("sigaro ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arp() {
    parse_logical_expr_ast("signed'(sigarp)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arq() {
    parse_logical_expr_ast("sigarq >= thresholdarq")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arr() {
    parse_logical_expr_ast("sigarr || readyarr").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ars() {
    parse_logical_expr_ast("sigars + 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_art() {
    parse_logical_expr_ast("(sigart & maskart) == 13")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aru() {
    parse_logical_expr_ast("sigaru[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arv() {
    parse_logical_expr_ast("sigarv[3:0] != 4'hf").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arw() {
    parse_logical_expr_ast("sigarw inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arx() {
    parse_logical_expr_ast("flagarx ? yesarx : noarx")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ary() {
    parse_logical_expr_ast("{2{sigary}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_arz() {
    parse_logical_expr_ast("sigarz << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asa() {
    parse_logical_expr_ast("sigasa >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asb() {
    parse_logical_expr_ast("~&sigasb").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asc() {
    parse_logical_expr_ast("^~sigasc").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asd() {
    parse_logical_expr_ast("type(stateasd)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ase() {
    parse_logical_expr_ast("logic[8]'(sigase)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asf() {
    parse_logical_expr_ast("unsigned'(sigasf)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asg() {
    parse_logical_expr_ast("sigasg.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ash() {
    parse_logical_expr_ast("{sigash, maskash} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asi() {
    parse_logical_expr_ast("sigasi ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asj() {
    parse_logical_expr_ast("signed'(sigasj)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ask() {
    parse_logical_expr_ast("sigask >= thresholdask")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asl() {
    parse_logical_expr_ast("sigasl || readyasl").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asm() {
    parse_logical_expr_ast("sigasm + 4").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asn() {
    parse_logical_expr_ast("(sigasn & maskasn) == 1")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aso() {
    parse_logical_expr_ast("sigaso[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asp() {
    parse_logical_expr_ast("sigasp[3:0] != 4'h3").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asq() {
    parse_logical_expr_ast("sigasq inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asr() {
    parse_logical_expr_ast("flagasr ? yesasr : noasr")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ass() {
    parse_logical_expr_ast("{2{sigass}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ast() {
    parse_logical_expr_ast("sigast << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asu() {
    parse_logical_expr_ast("sigasu >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asv() {
    parse_logical_expr_ast("~&sigasv").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asw() {
    parse_logical_expr_ast("^~sigasw").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asx() {
    parse_logical_expr_ast("type(stateasx)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asy() {
    parse_logical_expr_ast("logic[8]'(sigasy)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_asz() {
    parse_logical_expr_ast("unsigned'(sigasz)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ata() {
    parse_logical_expr_ast("sigata.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atb() {
    parse_logical_expr_ast("{sigatb, maskatb} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atc() {
    parse_logical_expr_ast("sigatc ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atd() {
    parse_logical_expr_ast("signed'(sigatd)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ate() {
    parse_logical_expr_ast("sigate >= thresholdate")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atf() {
    parse_logical_expr_ast("sigatf || readyatf").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atg() {
    parse_logical_expr_ast("sigatg + 7").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ath() {
    parse_logical_expr_ast("(sigath & maskath) == 5")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ati() {
    parse_logical_expr_ast("sigati[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atj() {
    parse_logical_expr_ast("sigatj[3:0] != 4'h7").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atk() {
    parse_logical_expr_ast("sigatk inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atl() {
    parse_logical_expr_ast("flagatl ? yesatl : noatl")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atm() {
    parse_logical_expr_ast("{2{sigatm}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atn() {
    parse_logical_expr_ast("sigatn << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ato() {
    parse_logical_expr_ast("sigato >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atp() {
    parse_logical_expr_ast("~&sigatp").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atq() {
    parse_logical_expr_ast("^~sigatq").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atr() {
    parse_logical_expr_ast("type(stateatr)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ats() {
    parse_logical_expr_ast("logic[8]'(sigats)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_att() {
    parse_logical_expr_ast("unsigned'(sigatt)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atu() {
    parse_logical_expr_ast("sigatu.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atv() {
    parse_logical_expr_ast("{sigatv, maskatv} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atw() {
    parse_logical_expr_ast("sigatw ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atx() {
    parse_logical_expr_ast("signed'(sigatx)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aty() {
    parse_logical_expr_ast("sigaty >= thresholdaty")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_atz() {
    parse_logical_expr_ast("sigatz || readyatz").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aua() {
    parse_logical_expr_ast("sigaua + 10").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aub() {
    parse_logical_expr_ast("(sigaub & maskaub) == 9")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_auc() {
    parse_logical_expr_ast("sigauc[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aud() {
    parse_logical_expr_ast("sigaud[3:0] != 4'hb").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aue() {
    parse_logical_expr_ast("sigaue inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_auf() {
    parse_logical_expr_ast("flagauf ? yesauf : noauf")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aug() {
    parse_logical_expr_ast("{2{sigaug}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_auh() {
    parse_logical_expr_ast("sigauh << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aui() {
    parse_logical_expr_ast("sigaui >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_auj() {
    parse_logical_expr_ast("~&sigauj").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_auk() {
    parse_logical_expr_ast("^~sigauk").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aul() {
    parse_logical_expr_ast("type(stateaul)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aum() {
    parse_logical_expr_ast("logic[8]'(sigaum)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aun() {
    parse_logical_expr_ast("unsigned'(sigaun)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_auo() {
    parse_logical_expr_ast("sigauo.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aup() {
    parse_logical_expr_ast("{sigaup, maskaup} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_auq() {
    parse_logical_expr_ast("sigauq ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aur() {
    parse_logical_expr_ast("signed'(sigaur)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aus() {
    parse_logical_expr_ast("sigaus >= thresholdaus")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aut() {
    parse_logical_expr_ast("sigaut || readyaut").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_auu() {
    parse_logical_expr_ast("sigauu + 13").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_auv() {
    parse_logical_expr_ast("(sigauv & maskauv) == 13")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_auw() {
    parse_logical_expr_ast("sigauw[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aux() {
    parse_logical_expr_ast("sigaux[3:0] != 4'hf").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_auy() {
    parse_logical_expr_ast("sigauy inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_auz() {
    parse_logical_expr_ast("flagauz ? yesauz : noauz")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ava() {
    parse_logical_expr_ast("{2{sigava}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avb() {
    parse_logical_expr_ast("sigavb << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avc() {
    parse_logical_expr_ast("sigavc >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avd() {
    parse_logical_expr_ast("~&sigavd").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_ave() {
    parse_logical_expr_ast("^~sigave").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avf() {
    parse_logical_expr_ast("type(stateavf)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avg() {
    parse_logical_expr_ast("logic[8]'(sigavg)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avh() {
    parse_logical_expr_ast("unsigned'(sigavh)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avi() {
    parse_logical_expr_ast("sigavi.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avj() {
    parse_logical_expr_ast("{sigavj, maskavj} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avk() {
    parse_logical_expr_ast("sigavk ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avl() {
    parse_logical_expr_ast("signed'(sigavl)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avm() {
    parse_logical_expr_ast("sigavm >= thresholdavm")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avn() {
    parse_logical_expr_ast("sigavn || readyavn").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avo() {
    parse_logical_expr_ast("sigavo + 16").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avp() {
    parse_logical_expr_ast("(sigavp & maskavp) == 1")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avq() {
    parse_logical_expr_ast("sigavq[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avr() {
    parse_logical_expr_ast("sigavr[3:0] != 4'h3").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avs() {
    parse_logical_expr_ast("sigavs inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avt() {
    parse_logical_expr_ast("flagavt ? yesavt : noavt")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avu() {
    parse_logical_expr_ast("{2{sigavu}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avv() {
    parse_logical_expr_ast("sigavv << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avw() {
    parse_logical_expr_ast("sigavw >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avx() {
    parse_logical_expr_ast("~&sigavx").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avy() {
    parse_logical_expr_ast("^~sigavy").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_avz() {
    parse_logical_expr_ast("type(stateavz)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awa() {
    parse_logical_expr_ast("logic[8]'(sigawa)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awb() {
    parse_logical_expr_ast("unsigned'(sigawb)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awc() {
    parse_logical_expr_ast("sigawc.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awd() {
    parse_logical_expr_ast("{sigawd, maskawd} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awe() {
    parse_logical_expr_ast("sigawe ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awf() {
    parse_logical_expr_ast("signed'(sigawf)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awg() {
    parse_logical_expr_ast("sigawg >= thresholdawg")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awh() {
    parse_logical_expr_ast("sigawh || readyawh").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awi() {
    parse_logical_expr_ast("sigawi + 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awj() {
    parse_logical_expr_ast("(sigawj & maskawj) == 5")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awk() {
    parse_logical_expr_ast("sigawk[0] == 1'b1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awl() {
    parse_logical_expr_ast("sigawl[3:0] != 4'h7").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awm() {
    parse_logical_expr_ast("sigawm inside {1, 2, 3}")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awn() {
    parse_logical_expr_ast("flagawn ? yesawn : noawn")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awo() {
    parse_logical_expr_ast("{2{sigawo}}").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awp() {
    parse_logical_expr_ast("sigawp << 1").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awq() {
    parse_logical_expr_ast("sigawq >>> 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awr() {
    parse_logical_expr_ast("~&sigawr").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aws() {
    parse_logical_expr_ast("^~sigaws").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awt() {
    parse_logical_expr_ast("type(stateawt)::IDLE")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awu() {
    parse_logical_expr_ast("logic[8]'(sigawu)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awv() {
    parse_logical_expr_ast("unsigned'(sigawv)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_aww() {
    parse_logical_expr_ast("sigaww.triggered()").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awx() {
    parse_logical_expr_ast("{sigawx, maskawx} == 2'b10")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awy() {
    parse_logical_expr_ast("sigawy ** 2").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_awz() {
    parse_logical_expr_ast("signed'(sigawz)").expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_axa() {
    parse_logical_expr_ast("sigaxa >= thresholdaxa")
        .expect("logical surface expression should parse");
}

#[test]
fn parser_surface_logical_axb() {
    parse_logical_expr_ast("sigaxb || readyaxb").expect("logical surface expression should parse");
}
