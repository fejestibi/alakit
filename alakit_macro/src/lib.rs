use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitStr};

/// Egyedi attribútum makró, amely regisztrálja a megadott struct-ot a központi
/// AlakitController regiszterbe (inventory) a megadott névterületen (namespace).
#[proc_macro_attribute]
pub fn alakit_controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Kinyerjük a namespace-t az attribútumból, pl: #[alakit_controller("settings")]
    let namespace = parse_macro_input!(attr as LitStr);

    // Parse-oljuk a magát a struct-ot, amire rátették
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    // Generáljuk a kimenetet: az eredeti struct + az inventory regisztráció
    let expanded = quote! {
        #input

        // Injektálunk egy inventory::submit! blokkot minden egyedi vezérlőhöz.
        inventory::submit! {
            alakit::ControllerRegistration {
                namespace: #namespace,
                factory: || Box::new(#struct_name::default()) as Box<dyn alakit::AlakitController + Send + Sync>,
            }
        }
    };

    TokenStream::from(expanded)
}
