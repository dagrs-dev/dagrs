use proc_macro2::TokenStream;
use syn::{
    Data, DeriveInput, Expr, Field, Fields, GenericArgument, Ident, Lit, Meta, PathArguments, Type,
    TypeParamBound,
};

const ID: &str = "id";
const NAME: &str = "name";
const PRECURSORS: &str = "precursors";
const ACTION: &str = "action";

enum Attr<'a> {
    Name(&'a Ident),
    Action(&'a Ident),
    Precursors(&'a Ident),
    Id(&'a Ident),
    Non,
}

#[allow(unused)]
pub fn parse_task(input: &DeriveInput) -> TokenStream {
    let struct_name = &input.ident;
    let mut id: Option<&Ident> = None;
    let mut name: Option<&Ident> = None;
    let mut precursors: Option<&Ident> = None;
    let mut action: Option<&Ident> = None;
    match input.data {
        Data::Struct(ref data) => {
            if let Fields::Named(ref named) = data.fields {
                named
                    .named
                    .iter()
                    .for_each(|field| match parse_attribute(field) {
                        Attr::Name(ident) => name = Some(ident),
                        Attr::Action(ident) => action = Some(ident),
                        Attr::Precursors(ident) => precursors = Some(ident),
                        Attr::Id(ident) => id = Some(ident),
                        Attr::Non => {}
                    })
            }
        }
        _ => {
            unimplemented!("`Task` can only be marked on struct.")
        }
    }
    if let [Some(id), Some(name), Some(precursors), Some(action)] = [id, name, precursors, action] {
        generate_impl(struct_name, id, name, precursors, action)
    } else {
        let example = "#[derive(Task)]\n".to_string()
            + "struct MyTask {\n"
            + "\t#[attr = \"id\"]\n"
            + "\tid: usize,\n"
            + "\t#[attr = \"name\"]\n"
            + "\tname: String,\n"
            + "\t#[attr = \"precursors\"]\n"
            + "\tpre: Vec<usize>,\n"
            + "\t#[attr = \"action\"]\n"
            + "\taction: Arc<dyn Action+Send+Sync>,\n"
            + "}";
        panic!(
            "Four required attributes must be marked with `attr`, for example:\n{}",
            example
        )
    }
}

const REQUIRED_ATTRS: &str =
    "`attr` value optional values: [\"id\",\"name\",\"action\",\"precursors\"]";

fn parse_attribute(field: &Field) -> Attr {
    let attrs = &field.attrs;
    let ty = &field.ty;
    for attr in attrs {
        let attr_ident = attr.path().get_ident();
        if attr_ident.is_some() && attr_ident.unwrap().eq("attr") {
            let ident = field.ident.as_ref().unwrap();
            if let Meta::NameValue(ref kv) = attr.meta {
                if let Expr::Lit(ref v) = kv.value {
                    match v.lit {
                        Lit::Str(ref attr_value) => match attr_value.value().as_str() {
                            ID => {
                                validate_id(ty);
                                return Attr::Id(ident);
                            }
                            NAME => {
                                validate_name(ty);
                                return Attr::Name(ident);
                            }
                            PRECURSORS => {
                                validate_precursors(ty);
                                return Attr::Precursors(ident);
                            }
                            ACTION => {
                                validate_action(ty);
                                return Attr::Action(ident);
                            }
                            _ => unimplemented!("{}", REQUIRED_ATTRS),
                        },
                        _ => {
                            unimplemented!("{}", REQUIRED_ATTRS)
                        }
                    }
                }
            }
        }
    }
    Attr::Non
}

const FIX_ID: &str = "The id field must be of type usize. id: usize";

fn validate_id(ty: &Type) {
    match ty {
        Type::Path(ref p) => {
            if !p.path.get_ident().unwrap().eq("usize") {
                unimplemented!("{}", FIX_ID)
            }
        }
        _ => {
            unimplemented!("{}", FIX_ID)
        }
    }
}

const FIX_NAME: &str = "The name field must be of type String. name: String";

fn validate_name(ty: &Type) {
    match ty {
        Type::Path(ref p) => {
            if !p.path.get_ident().unwrap().eq("String") {
                unimplemented!("{}", FIX_NAME)
            }
        }
        _ => {
            unimplemented!("{}", FIX_NAME)
        }
    }
}

const FIX_PRE: &str = "The precursors field must be of type Vec<usize>";

fn validate_precursors(ty: &Type) {
    match ty {
        Type::Path(ref p) => {
            let seg = &p.path.segments.first().unwrap();
            let generic_type = match seg.arguments {
                PathArguments::AngleBracketed(ref ab) => {
                    let pt = &ab.args;
                    let generic = pt.first().unwrap();
                    match generic {
                        GenericArgument::Type(ref t) => match t {
                            Type::Path(ref p) => &p.path.segments.first().unwrap().ident,
                            _ => unimplemented!(),
                        },
                        _ => unimplemented!("{}", FIX_PRE),
                    }
                }
                _ => unimplemented!("{}", FIX_PRE),
            };
            if !seg.ident.eq("Vec") || !generic_type.eq("usize") {
                unimplemented!("{}", FIX_PRE)
            }
        }
        _ => {
            unimplemented!("{}", FIX_PRE)
        }
    }
}

const FIX_ACTION: &str = "The action field must be of type Arc<Action+Sync+Send>";

fn validate_action(ty: &Type) {
    let mut v_action = false;
    let mut v_sync = false;
    let mut v_send = false;
    match ty {
        Type::Path(ref p) => {
            let seg = &p.path.segments.first().unwrap();
            match seg.arguments {
                PathArguments::AngleBracketed(ref ab) => match ab.args.first().unwrap() {
                    GenericArgument::Type(ref t) => match t {
                        Type::TraitObject(ref to) => {
                            let bounds = &to.bounds;
                            if bounds.len() != 3 {
                                unimplemented!("{}", FIX_ACTION)
                            }
                            bounds.into_iter().for_each(|bound| match bound {
                                TypeParamBound::Trait(tb) => {
                                    let ident = tb.path.get_ident().unwrap();
                                    match ident.to_string().as_str() {
                                        "Action" => v_action = true,
                                        "Sync" => v_sync = true,
                                        "Send" => v_send = true,
                                        _ => unimplemented!("{}", FIX_ACTION),
                                    }
                                }
                                _ => unimplemented!("{}", FIX_ACTION),
                            });
                        }
                        _ => {
                            unimplemented!("{}", FIX_ACTION)
                        }
                    },
                    _ => unimplemented!("{}", FIX_ACTION),
                },
                _ => unimplemented!("{}", FIX_ACTION),
            };
            if !seg.ident.eq("Arc") || !v_action || !v_send || !v_sync {
                unimplemented!("{}", FIX_ACTION)
            }
        }
        _ => {
            unimplemented!("{}", FIX_ACTION)
        }
    }
}

fn generate_impl(
    struct_name: &Ident,
    id: &Ident,
    name: &Ident,
    precursors: &Ident,
    action: &Ident,
) -> TokenStream {
    quote::quote!(
        impl Task for #struct_name{
            fn action(&self) -> Arc<dyn Action + Send + Sync>{
                self.#action.clone()
            }

            fn precursors(&self) -> &[usize]{
                &self.#precursors[..]
            }

            fn id(&self) -> usize{
                self.#id
            }

            fn name(&self) -> String{
                self.#name.clone()
            }
        }
        unsafe impl Send for #struct_name{}
        unsafe impl Sync for #struct_name{}
    )
}
