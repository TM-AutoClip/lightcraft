//! TM AutoClip yerel yaması — LightCraft (11 Eki 2026). Upstream'e dokunuş: `lib.rs` → `pub mod tm;` + her karede
//! `tm::kare`, `menus.rs` → çalıştırma, `menubar.rs` → marka linkleri, `panels/dialogs.rs` → Dışa Aktar penceresinde alan,
//! `apps/lightcraft/src/main.rs` → açılış kapısı.
//!
//! - Akış kipinde Dışa Aktar penceresinin sonunda "Dışa aktar ve TM AutoClip'e dön": penceredeki ayarlarla, oturumun
//!   beklenen çıktı klasörüne ve adıyla (biçimin uzantısıyla) dışa aktarır; iş bitince `teslim.json` yazar ve kapanır.
//! - Düz kipte alan GİZLİ. `craft-marka.json` linkleri (yalnız https) Yardım menüsünde Discord öğesinin YANINA eklenir.

pub mod oturum;

use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::{Value, json};

use crate::LightcraftApp;
use crate::menubar::MenuNode;

/// Bu programın oturumdaki adı.
pub const PROGRAM: &str = "lightcraft";
/// Marka linki komut öneki: `tm.link.<sıra>`.
const LINK: &str = "tm.link.";
/// "Dışa aktar ve dön" komutu (kontrol kanalından sınamak için de).
const DONUS: &str = "tm.disaAktarVeDon";

/// Bizim dışa aktarmamız sürüyor (bittiğinde teslim + kapanış).
static BEKLIYOR: AtomicBool = AtomicBool::new(false);

/// Yardım menüsüne marka linkleri (Discord öğesinden sonra).
pub fn menu_ogeleri(bar: &mut [(String, Vec<MenuNode>)]) {
    let Some(o) = oturum::simdiki() else { return };
    let Some((_, items)) = bar.iter_mut().find(|(t, _)| t == "Help") else { return };
    let Some(at) = items.iter().position(|n| matches!(n, MenuNode::Item { id, .. } if id == "app.discord")) else { return };
    for (k, l) in o.linkler.iter().enumerate() {
        let oge = MenuNode::Item { id: format!("{LINK}{k}"), params: Value::Null, label: l.etiket.clone(), shortcut: None, enabled: true, checked: None };
        items.insert(at + 1 + k, oge);
    }
}

/// Bizim komutlar (`None` = bizim değil). `DONUS` parametresi Dışa Aktar penceresinin ayarlarıdır (`app.export` biçimi).
pub fn calistir(app: &mut LightcraftApp, id: &str, p: &Value) -> Option<Result<Value, String>> {
    if let Some(sira) = id.strip_prefix(LINK) {
        let adres = oturum::simdiki().and_then(|o| sira.parse::<usize>().ok().and_then(|k| o.linkler.get(k))).map(|l| l.adres.clone());
        return Some(match adres {
            Some(a) => crate::links::open(app, &a),
            None => Err("bilinmeyen link".into()),
        });
    }
    (id == DONUS).then(|| baslat(app, p.clone()))
}

/// Dışa Aktar penceresinin sonunda bizim alan (yalnız akış kipinde). `ayarlar`: penceredeki seçimler (`app.export` biçimi).
pub fn disa_aktar_alani(app: &mut LightcraftApp, ui: &mut egui::Ui, ayarlar: impl FnOnce() -> Value) {
    let Some(o) = oturum::akis() else { return };
    ui.add_space(6.0);
    ui.separator();
    let etiket = o.metin("disaAktarDonus", "Dışa aktar ve TM AutoClip'e dön").to_string();
    let suruyor = BEKLIYOR.load(Ordering::Relaxed);
    let r = ui.add_enabled(!suruyor, egui::Button::new(egui::RichText::new(&etiket).strong()));
    crate::widgets::register(ui.ctx(), "button:tmDisaAktarVeDon", r.rect);
    if let Some((_, yol)) = &o.cikti {
        ui.label(egui::RichText::new(yol.to_string_lossy()).small().weak());
    }
    if r.clicked() {
        let ctx = ui.ctx().clone();
        match baslat(app, ayarlar()) {
            Ok(_) => app.toast(&ctx, "TM AutoClip için dışa aktarılıyor…"),
            Err(e) => app.toast(&ctx, e),
        }
    }
}

/// Dışa aktarmayı oturumun çıktı klasörüne/adına başlatır (arka planda; bitişini `kare` izler).
fn baslat(app: &mut LightcraftApp, mut p: Value) -> Result<Value, String> {
    let o = oturum::akis().ok_or("TM AutoClip akış oturumu yok")?;
    if BEKLIYOR.load(Ordering::Relaxed) || app.export.is_some() {
        return Err("bir dışa aktarma zaten sürüyor".into());
    }
    let (_, yol) = o.cikti.clone().ok_or("oturumda çıktı yolu yok")?;
    let klasor = yol.parent().ok_or("çıktı yolunun klasörü yok")?;
    let ad = yol.file_stem().map(|s| s.to_string_lossy().into_owned()).ok_or("çıktı yolunun adı yok")?;
    if !p.is_object() {
        p = json!({});
    }
    p["dir"] = json!(klasor.to_string_lossy());
    p["subfolder"] = json!("");
    p["naming"] = json!(ad);
    p["background"] = json!(true);
    app.last_export_result = None;
    let v = app.run("app.export", p).map_err(|e| {
        log::error!("TM AutoClip: dışa aktarma başlamadı: {e}");
        e
    })?;
    if app.export.is_some() {
        BEKLIYOR.store(true, Ordering::Relaxed);
    } else {
        // Arka plan yazıcısı yoksa eşzamanlı bitti: sonucu hemen işle.
        BEKLIYOR.store(true, Ordering::Relaxed);
        app.last_export_result = app.last_export_result.take().or_else(|| Some(json!({"files": v.get("files").cloned().unwrap_or(v.clone())})));
    }
    log::info!("TM AutoClip: dışa aktarma başladı ({})", klasor.display());
    Ok(v)
}

/// Her kare: bizim dışa aktarmamız bittiyse ilk dosyayı teslim eder ve kapanır.
pub fn kare(app: &mut LightcraftApp, ctx: &egui::Context) {
    if !BEKLIYOR.load(Ordering::Relaxed) {
        return;
    }
    if app.export.is_some() {
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
        return;
    }
    BEKLIYOR.store(false, Ordering::Relaxed);
    let sonuc = app.last_export_result.clone().unwrap_or(Value::Null);
    let ilk = sonuc["files"].as_array().and_then(|a| a.iter().find_map(|f| f.get("path").and_then(Value::as_str).map(str::to_string)));
    let iptal = sonuc["cancelled"].as_bool() == Some(true);
    let (Some(yol), false) = (ilk, iptal) else {
        log::error!("TM AutoClip: dışa aktarma dosya üretmedi ({sonuc})");
        app.toast(ctx, "TM AutoClip'e gönderilemedi: dışa aktarma dosya üretmedi");
        return;
    };
    let Some(o) = oturum::akis() else { return };
    match oturum::teslim_yaz(o, std::path::Path::new(&yol)) {
        Ok(_) => {
            app.quit_confirmed = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        Err(e) => {
            log::error!("TM AutoClip: {e}");
            app.toast(ctx, e);
        }
    }
}
