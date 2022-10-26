extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn make_answer(_item: TokenStream) -> TokenStream {
    // pub static ref a: SyncUnsafeCell<bool> = SyncUnsafeCell::new(false);

    (
      "pub static ref ".to_string() 
      + &_item.to_string()
      + ": SyncUnsafeCell<bool> = SyncUnsafeCell::new(false);"
      ).parse().unwrap()
}
