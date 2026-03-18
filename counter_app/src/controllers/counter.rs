use alakit::{AlakitController, AppContext};
use alakit_macro::alakit_controller;

#[alakit_controller("counter")]
#[derive(Default)]
pub struct CounterController;

#[async_trait::async_trait]
impl AlakitController for CounterController {
    async fn handle(&self, command: &str, _args: &str, ctx: AppContext) {
        // Get current value from the Store
        let current_count: i32 = ctx.store.get("count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        match command {
            "increment" => {
                ctx.store.set("count", &(current_count + 1).to_string());
                ctx.dom.log(&format!("Counter incremented: {}", current_count + 1));
            },
            "decrement" => {
                ctx.store.set("count", &(current_count - 1).to_string());
                ctx.dom.log(&format!("Counter decremented: {}", current_count - 1));
            },
            "reset" => {
                ctx.store.set("count", "0");
                ctx.dom.toast_info("Counter has been reset");
                ctx.dom.log("Counter reset");
            },
            _ => println!("Unknown command from Counter: {}", command),
        }
    }
}
