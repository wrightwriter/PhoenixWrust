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

#[proc_macro]
pub fn add_uniform(s: TokenStream) -> TokenStream {
    let stream_str = s.to_string();
    let stream_str = stream_str.replace(" ", "");

    let args: Vec<_> = stream_str.split(",").into_iter().collect();
    let arg_name = args[2];
    let arg_type = args[1];
    let arg_idx = args[0];
    // let arg_val = args[2];

    assert!(args.len() == 3);

    (
      "pub fn setUniform".to_string() 
      + &arg_name + 
      "(&mut self, val: " + arg_type + ")" + 
      "{{ self.get_uniforms_container().set_at( " + &arg_idx + ", val ); }}"
  ).parse().unwrap()
}


#[proc_macro]
pub fn init_uniform(s: TokenStream) -> TokenStream {
    let stream_str = s.to_string();
    let stream_str = stream_str.replace(" ", "");

    let args: Vec<_> = stream_str.split(",").into_iter().collect();
    let arg_struct = args[0];
    let arg_name = args[1];
    let arg_value = args[2];

    assert!(args.len() == 3);

    (
      "".to_string() + &arg_struct + 
      ".setUniform".into()
      + &arg_name + 
      "(" + arg_value + ");
      " + &arg_struct + ".get_uniforms_container().uniforms_names.push(\"".into() + &arg_name + "\".into());".into()
      
  ).parse().unwrap()
}
