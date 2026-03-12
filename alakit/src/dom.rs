use tao::event_loop::EventLoopProxy;

/// Egy vékony, felhasználóbarát wrapper (burkoló) a JavaScript `evaluate_script`
/// hívások futtatásához biztonságosan, a Rust szálakon keresztül.
#[derive(Clone)]
pub struct RustDOM {
    // Küldünk string-formátumú JS kódokat a főszálnak futtatásra.
    pub proxy: EventLoopProxy<String>,
}

impl RustDOM {
    /// Lekér egy virtuális DOM elemet az ID-ja alapján
    pub fn get_element_by_id(&self, id: &str) -> Element {
        Element {
            id: id.to_string(),
            proxy: self.proxy.clone(),
        }
    }

    /// Rövidebb alias a get_element_by_id-hez
    pub fn get_id(&self, id: &str) -> Element {
        self.get_element_by_id(id)
    }

    /// Típusos Toast értesítés küldése
    fn send_toast(&self, type_str: &str, message: &str) {
        let safe_msg = message.replace('`', "\\`").replace('$', "\\$");
        let js = format!(
            "if (window.alakit_toast) window.alakit_toast('{}', `{}`);",
            type_str, safe_msg
        );
        let _ = self.proxy.send_event(js);
    }

    /// Pozitív (Zöld) visszajelzés
    pub fn toast_success(&self, message: &str) {
        self.send_toast("success", message);
    }

    /// Hiba (Piros) visszajelzés
    pub fn toast_error(&self, message: &str) {
        self.send_toast("error", message);
    }

    /// Figyelmeztetés (Sárga) visszajelzés
    pub fn toast_warning(&self, message: &str) {
        self.send_toast("warning", message);
    }

    /// Információs (Kék) visszajelzés
    pub fn toast_info(&self, message: &str) {
        self.send_toast("info", message);
    }

    /// Fejlesztői log küldése a böngésző konzoljára
    pub fn log(&self, message: &str) {
        let js = format!("console.log(`[RUST] {}`);", message.replace('`', "\\`"));
        let _ = self.proxy.send_event(js);
    }
}

/// Egy virtuális HTML elem, ami megőrzi a célzott azonosítót és képes manipuláló parancsokat küldeni rá.
#[derive(Clone)]
pub struct Element {
    pub id: String,
    pub proxy: EventLoopProxy<String>,
}

impl Element {
    pub fn set_text(&self, text: &str) -> &Self {
        let js = format!(
            "document.getElementById('{}').innerText = `{}`;",
            self.id,
            text.replace('`', "\\`")
        );
        let _ = self.proxy.send_event(js);
        self
    }

    pub fn set_value(&self, val: &str) -> &Self {
        let js = format!(
            "document.getElementById('{}').value = `{}`;",
            self.id,
            val.replace('`', "\\`")
        );
        let _ = self.proxy.send_event(js);
        self
    }

    pub fn set_style(&self, property: &str, value: &str) -> &Self {
        let js = format!(
            "document.getElementById('{}').style['{}'] = `{}`;",
            self.id,
            property,
            value.replace('`', "\\`")
        );
        let _ = self.proxy.send_event(js);
        self
    }

    pub fn add_class(&self, name: &str) -> &Self {
        let js = format!(
            "document.getElementById('{}').classList.add('{}');",
            self.id, name
        );
        let _ = self.proxy.send_event(js);
        self
    }

    pub fn remove_class(&self, name: &str) -> &Self {
        let js = format!(
            "document.getElementById('{}').classList.remove('{}');",
            self.id, name
        );
        let _ = self.proxy.send_event(js);
        self
    }

    /// Új attribútum hozzáadása az elemhez (pl. disabled, placeholder)
    pub fn set_attribute(&self, name: &str, value: &str) -> &Self {
        let js = format!(
            "document.getElementById('{}').setAttribute('{}', `{}`);",
            self.id,
            name,
            value.replace('`', "\\`")
        );
        let _ = self.proxy.send_event(js);
        self
    }

    /// Attribútum törlése az elemről
    pub fn remove_attribute(&self, name: &str) -> &Self {
        let js = format!(
            "document.getElementById('{}').removeAttribute('{}');",
            self.id, name
        );
        let _ = self.proxy.send_event(js);
        self
    }

    /// Nyers HTML dinamikus beillesztése az elem belsejébe (hozzáfűzés)
    pub fn append_html(&self, html: &str) -> &Self {
        let js = format!(
            "var el = document.getElementById('{}'); if(el) el.insertAdjacentHTML('beforeend', `{}`);",
            self.id,
            html.replace('`', "\\`")
        );
        let _ = self.proxy.send_event(js);
        self
    }

    /// Az elem belső HTML tartalmának teljes felülírása
    pub fn set_html(&self, html: &str) -> &Self {
        let js = format!(
            "var el = document.getElementById('{}'); if(el) el.innerHTML = `{}`;",
            self.id,
            html.replace('`', "\\`")
        );
        let _ = self.proxy.send_event(js);
        self
    }

    /// Elem teljes eltávolítása a DOM-ból
    pub fn remove(&self) {
        let js = format!(
            "var el = document.getElementById('{}'); if(el) el.remove();",
            self.id
        );
        let _ = self.proxy.send_event(js);
    }
}
