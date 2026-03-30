use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr, parse_macro_input};

/// A procedural macro that automatically implements `From<Enum> for u32` for instruction enums.
#[proc_macro_derive(InstructionEnum)]
pub fn derive_instruction_enum(input: TokenStream) -> TokenStream {
    // Parse the incoming Rust code into an Abstract Syntax Tree
    let input = parse_macro_input!(input as DeriveInput);

    // The name of the enum (e.g., "Instruction")
    let name = &input.ident;

    // Ensure this is only applied to Enums
    let syn::Data::Enum(data) = &input.data else {
        panic!("InstructionEnum can only be applied to enums");
    };

    let mut to_u32_arms = Vec::new();

    // Iterate over the variants (e.g., DataProcessing(DataProcessingInstruction))
    for variant in &data.variants {
        let variant_name = &variant.ident;

        // Extract the inner type (e.g., DataProcessingInstruction)
        let field_type = match &variant.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                &fields.unnamed.first().unwrap().ty
            }
            _ => panic!(
                "InstructionEnum variants must have exactly one unnamed field"
            ),
        };
        _ = field_type;

        // Generate the match arm for: Instruction::DataProcessing(inner) => inner.into()
        to_u32_arms.push(quote! {
            #name::#variant_name(inner) => inner.into(),
        });
    }

    // Combine all the generated code into the final output
    let expanded = quote! {
        impl From<#name> for u32 {
            fn from(inst: #name) -> Self {
                match inst {
                    #(#to_u32_arms)*
                }
            }
        }
    };

    // Hand the generated code back to the Rust compiler
    TokenStream::from(expanded)
}

/// Automatically generates the `TryFrom<u32>` instruction decoder.
/// It reads visual binary patterns from `#[decode("...")]` attributes,
/// computing the bitwise masks and values at compile time!
#[proc_macro_derive(ArmDecoder, attributes(decode))]
pub fn derive_arm_decoder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    let syn::Data::Enum(data) = &input.data else {
        panic!("ArmDecoder can only be applied to enums");
    };

    let mut checks = Vec::new();

    // Iterate through the variants top-to-bottom
    for variant in &data.variants {
        let variant_name = &variant.ident;

        for attr in &variant.attrs {
            // Find the #[decode("...")] attribute
            if attr.path().is_ident("decode") {
                let lit: LitStr = attr.parse_args().expect(
                    "Expected string literal in #[decode(\"...\")]",
                );
                let pattern = lit.value();

                let mut mask: u32 = 0;
                let mut value: u32 = 0;
                let mut bit_count = 0;

                // Parse the visual pattern
                for c in pattern.chars() {
                    match c {
                        '0' => {
                            mask |= 1 << (31 - bit_count);
                            bit_count += 1;
                        }
                        '1' => {
                            mask |= 1 << (31 - bit_count);
                            value |= 1 << (31 - bit_count);
                            bit_count += 1;
                        }
                        c if c.is_alphanumeric() || c == '-' => {
                            // Variable bits (like 'cond', 'Rn', 'S') don't affect the mask
                            bit_count += 1;
                        }
                        _ => {} // Ignore spaces, underscores, pipes
                    }
                }

                if bit_count != 32 {
                    panic!(
                        "Pattern '{}' for variant {} does not have exactly 32 bits (found {})",
                        pattern, variant_name, bit_count
                    );
                }

                // Generate the if-statement for this specific pattern
                checks.push(quote! {
                    if (raw_instruction & #mask) == #value {
                        return Ok(#enum_name::#variant_name(raw_instruction.into()));
                    }
                });
            }
        }
    }

    // Generate the final TryFrom implementation
    let expanded = quote! {
        impl TryFrom<u32> for #enum_name {
            type Error = crate::instructions::InstructionConversionError;

            fn try_from(raw_instruction: u32) -> Result<Self, Self::Error> {
                // The macro injects all the generated if-statements here
                #(#checks)*

                Err(crate::instructions::InstructionConversionError::InvalidInstructionClass)
            }
        }
    };

    TokenStream::from(expanded)
}
