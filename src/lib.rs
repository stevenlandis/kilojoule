#![feature(sized_type_properties)]
#![feature(alloc_layout_extra)]
#![feature(ptr_as_ref_unchecked)]

mod ast_node;
mod byte_vec;
mod evaluator;
mod json_lexer;
mod object_collector;
mod parser;
mod reader;
mod run_repl;
mod val;

// External exports
pub use evaluator::EvalCtx;
pub use json_lexer::{JsonLexer, JsonLexerTrait, JsonToken};
pub use reader::ReaderTrait;
pub use run_repl::run_repl;
