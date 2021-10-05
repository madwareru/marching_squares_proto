use {
    syn::{
        parse_macro_input, LitInt, punctuated::Punctuated, Token,
        parse::{ParseStream}
    },
};
use syn::parse_quote::ParseQuote;

#[derive(Clone)]
enum IndicesPart {
    Triangle(LitInt, LitInt, LitInt),
    Quad(LitInt, LitInt, LitInt, LitInt)
}

fn skip_token(input: &ParseStream) -> syn::Result<()> {
    input.step(|cursor| {
        let rest = *cursor;
        if let Some((_, next)) = rest.token_tree() {
            Ok(((), next))
        } else {
            Err(cursor.error(""))
        }
    })
}

impl syn::parse::Parse for IndicesPart {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![+]) {
            skip_token(&input)?;
            let idx_0: LitInt = syn::parse::Parse::parse(input)?;
            let idx_1: LitInt = syn::parse::Parse::parse(input)?;
            let idx_2: LitInt = syn::parse::Parse::parse(input)?;
            Ok(IndicesPart::Triangle(idx_0, idx_1, idx_2))
        } else if lookahead.peek(Token![*]) {
            skip_token(&input)?;
            let idx_0: LitInt = syn::parse::Parse::parse(input)?;
            let idx_1: LitInt = syn::parse::Parse::parse(input)?;
            let idx_2: LitInt = syn::parse::Parse::parse(input)?;
            let idx_3: LitInt = syn::parse::Parse::parse(input)?;
            Ok(IndicesPart::Quad(idx_0, idx_1, idx_2, idx_3))
        } else {
            Err(lookahead.error())
        }
    }
}

struct IndexParts {
    components: Vec<IndicesPart>
}
impl syn::parse::Parse for IndexParts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let components = Punctuated::<IndicesPart, Token![;]>::parse(input)?
            .iter()
            .map(|it| it.clone())
            .collect::<Vec<_>>();
        Ok(IndexParts{ components })
    }
}

#[proc_macro]
pub fn make_indices(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let IndexParts { components } = parse_macro_input!(input as IndexParts);
    let mut result_string = String::new();
    result_string += "[\n";
    for comp in components {
        match comp {
            IndicesPart::Triangle(id_0, id_1, id_2) => {
                result_string += &format!(
                    "    start_id + {}, start_id + {}, start_id + {},\n",
                    id_0.to_string(), id_1.to_string(), id_2.to_string()
                );
            }
            IndicesPart::Quad(id_0, id_1, id_2, id_3) => {
                result_string += &format!(
                    "    start_id + {}, start_id + {}, start_id + {},\n",
                    id_0.to_string(), id_1.to_string(), id_2.to_string()
                );
                result_string += &format!(
                    "    start_id + {}, start_id + {}, start_id + {},\n",
                    id_2.to_string(), id_1.to_string(), id_3.to_string()
                );
            }
        }
    }
    result_string += "]";
    (&result_string).parse::<proc_macro::TokenStream>().unwrap()
}
