use async_trait::async_trait;

/// Az alkalmazás kontextusa, amelyet a Rust vezérlők kapnak meg
/// a `handle` meghívásakor. Biztosítja az aszinkron hozzáférést a DOM-hoz.
#[derive(Clone)]
pub struct AppContext {
    pub dom: crate::dom::RustDOM,
    pub store: crate::store::Store,
}

#[async_trait]
pub trait AlakitController: Send + Sync {
    async fn handle(&self, cmd: &str, args: &str, ctx: AppContext);
    
    #[allow(unused_variables)]
    async fn handle_binary(&self, cmd: &str, payload: &[u8], ctx: AppContext) {
        // Alapértelmezésben nem csinál semmit. Írd felül, ha a vezérlőd támogatja a bináris adatokat.
    }
}

// Ez a struktúra fogja tárolni a generált adatokat a makróból
pub struct ControllerRegistration {
    pub namespace: &'static str,
    // Egy függvény mutató (factory), amivel létrehozható az adott kontroller
    pub factory: fn() -> Box<dyn AlakitController + Send + Sync>,
}

inventory::collect!(ControllerRegistration);
