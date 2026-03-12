
/// Az alkalmazás kontextusa, amelyet a Rust vezérlők kapnak meg
/// a `handle` meghívásakor. Biztosítja az aszinkron hozzáférést a DOM-hoz.
pub struct AppContext {
    pub dom: crate::dom::RustDOM,
    pub store: crate::store::Store,
}

pub trait AlakitController: Send + Sync {
    fn handle(&self, cmd: &str, args: &str, ctx: &AppContext);
}

// Ez a struktúra fogja tárolni a generált adatokat a makróból
pub struct ControllerRegistration {
    pub namespace: &'static str,
    // Egy függvény mutató (factory), amivel létrehozható az adott kontroller
    pub factory: fn() -> Box<dyn AlakitController>,
}

inventory::collect!(ControllerRegistration);
