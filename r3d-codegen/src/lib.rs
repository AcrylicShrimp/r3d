mod handles;

use proc_macro::TokenStream;
use proc_macro_error::*;

#[proc_macro_derive(Handle)]
#[proc_macro_error]
pub fn handle(item: TokenStream) -> TokenStream {
    handles::handle(item)
}

#[proc_macro_derive(HandleMut)]
#[proc_macro_error]
pub fn handle_mut(item: TokenStream) -> TokenStream {
    handles::handle_mut(item)
}
