use proc_macro::TokenStream;

#[cfg(feature = "derive")]
mod auto_node;

/// [`auto_node`] is a macro that may be used when customizing nodes. It can only be
/// marked on named struct or unit struct.
///
/// The macro [`auto_node`] generates essential fields and implementation of traits for
/// structs intended to represent `Node` in **Dagrs**.
/// By applying this macro to a struct, it appends fields including `id: dagrs::NodeId`,
/// `name: dagrs::NodeName`, `input_channels: dagrs::InChannels`, `output_channels: dagrs::OutChannels`,
/// and `action: dagrs::Action`, and implements the required `dagrs::Node` trait.
///
/// ## Example
/// - Mark `auto_node` on a struct with customized fields.
/// ```ignore
/// use dagrs::auto_node;
/// #[auto_node]
/// struct MyNode {/*Put your customized fields here.*/}
/// ```
///
/// - Mark `auto_node` on a struct with generic & lifetime params.
/// ```ignore
/// use dagrs::auto_node;
/// #[auto_node]
/// struct MyNode<T, 'a> {/*Put your customized fields here.*/}
/// ```
/// - Mark `auto_node` on a unit struct.
/// ```ignore
/// use dagrs::auto_node;
/// #[auto_node]
/// struct MyNode()
/// ```
#[cfg(feature = "derive")]
#[proc_macro_attribute]
pub fn auto_node(args: TokenStream, input: TokenStream) -> TokenStream {
    use crate::auto_node::auto_node;
    auto_node(args, input).into()
}
