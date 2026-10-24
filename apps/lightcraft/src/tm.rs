//! TM AutoClip yerel yaması — açılış kapısı (11 Eki 2026). Bu yamalı ikili yalnız TM AutoClip'ten açılır:
//! oturum (`TMAC_OTURUM` + `TMAC_BELIRTEC`) yoksa ya da geçersizse ileti gösterip kapanır. Denetim
//! `lightcraft_ui_egui::tm::oturum` içinde (üç yamalı programda ortak); ayrıntı TM AutoClip deposunda
//! `harici/craft-yamalar/YAMALAR.md`.

/// Oturum geçerli değilse kullanıcıya söyler ve süreci 3 koduyla kapatır.
pub fn kapi() {
    use lightcraft_ui_egui::tm::{PROGRAM, oturum};
    if let Err(sebep) = oturum::ac(PROGRAM) {
        log::warn!("TM AutoClip oturumu reddedildi: {sebep}");
        let metin = format!("{}\n\n({sebep})", oturum::RET_ILETISI);
        eprintln!("lightcraft: {metin}");
        let _ = rfd::MessageDialog::new().set_level(rfd::MessageLevel::Warning).set_title("LightCraft — TM AutoClip").set_description(&metin).show();
        std::process::exit(3);
    }
}
