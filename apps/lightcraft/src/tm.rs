//! TM AutoClip yerel yaması — açılış kapısı (11 Eki 2026; canlı uyumu 3 Kas 2026). Oturum (`TMAC_OTURUM` +
//! `TMAC_BELIRTEC`) hiç verilmediyse program serbest açılır (canlıda TM AutoClip menüsünden düz açılış); oturum verildiyse
//! ve geçersizse ileti gösterip kapanır. Denetim
//! `lightcraft_ui_egui::tm::oturum` içinde (üç yamalı programda ortak); ayrıntı TM AutoClip deposunda
//! `harici/craft-yamalar/YAMALAR.md`.

/// Verilen oturum geçerli değilse kullanıcıya söyler ve süreci 3 koduyla kapatır; oturum yoksa bir şey yapmaz.
pub fn kapi() {
    use lightcraft_ui_egui::tm::{PROGRAM, oturum};
    if let Err(sebep) = oturum::kapi_karari(PROGRAM) {
        log::warn!("TM AutoClip oturumu reddedildi: {sebep}");
        let metin = format!("{}\n\n({sebep})", oturum::RET_ILETISI);
        eprintln!("lightcraft: {metin}");
        let _ = rfd::MessageDialog::new().set_level(rfd::MessageLevel::Warning).set_title("LightCraft — TM AutoClip").set_description(&metin).show();
        std::process::exit(3);
    }
}
