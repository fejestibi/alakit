use alakit::{AlakitController, AppContext};
use alakit_macro::alakit_controller;
use serde_json::json;

#[derive(Default)]
#[alakit_controller("vault")]
pub struct VaultController;

impl AlakitController for VaultController {
    fn handle(&self, command: &str, args: &str, ctx: &AppContext) {
        match command {
            "unlock" => self.unlock_vault(ctx, args),
            "lock" => self.lock_vault(ctx),
            _ => {
                ctx.dom
                    .toast_warning(&format!("Unknown vault command: {}", command));
            }
        }
    }
}

impl VaultController {
    fn unlock_vault(&self, ctx: &AppContext, args: &str) {
        // 1. Get the master password from the JSON sent by alakit-form
        let data: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let master_pwd = data["master-password"].as_str().unwrap_or("");

        // 2. Simulated check
        if master_pwd == "alakit123" {
            // SUCCESSFUL LOGIN:
            // 3. Load data into the AES Encrypted Store for "Production" simulation.
            self.load_secured_credentials(ctx);

            // 4. State switch: Set the Vault to unlocked state
            ctx.store.set("vault_unlocked", "true");
            ctx.store.set("vault_locked", "false");

            // 5. Send Toast
            ctx.dom
                .toast_success("Vault Unlocked! AES-256 Memory Activated.");

            // 6. Cleanup: Clear the password input for security
            ctx.dom.get_element_by_id("master-password").set_value("");
        } else {
            // WRONG PASSWORD
            ctx.dom
                .toast_error("Access Denied: Invalid Master Password.");
        }
    }

    fn lock_vault(&self, ctx: &AppContext) {
        // LOCKING
        // Clear everything from the AES Store to maintain security
        ctx.store.remove("vault_data");

        // Close Reactive UI
        ctx.store.set("vault_unlocked", "false");
        ctx.store.set("vault_locked", "true");

        ctx.dom.toast_info("Vault Locked. Memory cleared.");
    }

    fn load_secured_credentials(&self, ctx: &AppContext) {
        let creds = vec![
            json!({"service": "Netflix", "user": "felix@alakit.rs", "pass": "N3tfl!x_Rust"}),
            json!({"service": "Github", "user": "RustHero", "pass": "g1t_c0mm1t_pusH"}),
            json!({"service": "Banking App", "user": "admin", "pass": "$uperS3cr3tB@nk!"}),
        ];

        // Put into the Crypt Store. In memory, this is now just binary AES blob:
        let stringified_creds = serde_json::to_string(&creds).unwrap();
        ctx.store.set("vault_data", &stringified_creds);

        // Render to HTML using the DOM Builder
        let mut html_cards = String::new();
        for item in creds {
            let srv = item["service"].as_str().unwrap();
            let usr = item["user"].as_str().unwrap();
            let pwd = item["pass"].as_str().unwrap();

            // Link the JavaScript clipboard copy call
            html_cards.push_str(&format!(
                r#"
                <div class="pwd-item">
                    <div class="pwd-info">
                        <h3>{}</h3>
                        <p>{}</p>
                    </div>
                    <button class="icon-btn copy-btn" title="Copy" onclick="window.copyToClipboard('{}')">
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="var(--accent)" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                    </button>
                </div>
                "#,
                srv, usr, pwd
            ));
        }

        ctx.dom
            .get_element_by_id("password-list")
            .set_html(&html_cards);
    }
}
