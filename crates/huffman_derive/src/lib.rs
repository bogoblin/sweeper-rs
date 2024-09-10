use crate::HuffmanTree::{Leaf, Tree};
use proc_macro::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::{Display, Formatter};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, ExprPath, ItemEnum, LitFloat, LitInt, PatPath, Token};

enum HuffmanTree {
    Tree {
        left: Box<HuffmanTree>,
        right: Box<HuffmanTree>,
        probability: f64,
    },
    Leaf(ExprPath, f64)
}

struct InverseHuffmanTree(Vec<(ExprPath, Vec<bool>)>);

impl Display for HuffmanTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Tree { left, right, probability } => {
                f.write_str(format!("({probability}: [{left}, {right}]").as_str())
            }
            Leaf(expr_path, probability) => {
                let expr_path = expr_path.into_token_stream().to_string();
                f.write_str(format!("({probability}: ({expr_path}))").as_str())
            }
        }
    }
}

impl HuffmanTree {
    fn probability(&self) -> f64 {
        match self {
            Tree { left, right, probability } => *probability,
            Leaf(_, probability) => *probability,
        }
    }
    
    fn inverse(&self) -> InverseHuffmanTree {
        InverseHuffmanTree(self._inverse(vec![]))
    }
    
    fn _inverse(&self, prefix: Vec<bool>) -> Vec<(ExprPath, Vec<bool>)> {
        match self {
            Tree { left, right, probability } => {
                let mut with_true = prefix.clone(); with_true.push(true);
                let mut with_false = prefix.clone(); with_false.push(false);
                let mut result = left._inverse(with_true);
                result.append(&mut right._inverse(with_false));
                result
            },
            Leaf(expr_path, _) => {
                vec![(expr_path.clone(), prefix)]
            }
        }
    }
}

impl ToTokens for InverseHuffmanTree {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let keys: Vec<_> = self.0.iter().map(|e| &e.0).collect();
        let values: Vec<_> = self.0.iter().map(|e| &e.1).collect();

        tokens.append_all(
            quote! { match self {
                #(#keys => vec![#(#values),*]),*
            } }
        )
    }
}

impl ToTokens for HuffmanTree {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let stream = match self {
            Tree { left, right, probability } => {
                quote! { if decode_from.read_byte()? {
                    #left 
                } else {
                    #right 
                }}
            },
            Leaf(path, _probability) => {
                quote! { Some(Box::new(#path)) }
            }
        };
        tokens.append_all(stream)
    }
}

impl Eq for HuffmanTree {}

impl PartialEq<Self> for HuffmanTree {
    fn eq(&self, other: &Self) -> bool {
        self.probability() == other.probability()
    }
}

impl PartialOrd<Self> for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HuffmanTree {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp = self.probability() - other.probability();
        if cmp <= 0f64 { // Sorted backwards so we have a min-heap
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl Parse for HuffmanTree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // let type_of: Type = input.parse().expect("not the type");
        let mut heap = BinaryHeap::new();
        loop {
            let thing = input.parse::<PatPath>().expect("1");
            let _arrow = input.parse::<Token![=>]>().expect("2");
            let probability: f64 = if input.peek(LitFloat) {
                input.parse::<LitFloat>().expect("3").base10_parse().expect("couldn't parse float")
            } else if input.peek(LitInt){
                input.parse::<LitInt>().expect("not an int").base10_parse().expect("couldn't parse int")
            } else {
                panic!("expected a number");
            };
            heap.push(Leaf(thing, probability));
            if input.parse::<Token![,]>().is_err() {
                break;
            }
        }

        loop {
            let n1 = heap.pop().expect("Expected nodes remaining");
            if let Some(n2) = heap.pop() {
                let probability = n1.probability() + n2.probability();
                heap.push(Tree {
                    left: Box::from(n1),
                    right: Box::from(n2),
                    probability,
                })
            } else {
                return Ok(n1);
            }
        }
    }
}

#[proc_macro_attribute]
pub fn huffman_derive(attr: TokenStream, input: TokenStream) -> TokenStream {
    let tree = parse_macro_input!(attr as HuffmanTree);
    let inverted = tree.inverse();

    let orig_enum = input.clone();

    let our_enum = parse_macro_input!(input as ItemEnum);
    let type_of = our_enum.ident;

    TokenStream::from_iter(vec![
        quote! {
        use huffman::{BitReader, BitWriter};
        impl HuffmanCode for #type_of {
            fn encode(&self, encode_to: &mut BitWriter) {
                let encoded = #inverted;
                for bit in encoded {
                    encode_to.write_bit(bit);
                }
            }
            fn decode(decode_from: &mut BitReader) -> Option<Box<Self>> {
                #tree
            }
        } }.into(),
        orig_enum
    ])
}
