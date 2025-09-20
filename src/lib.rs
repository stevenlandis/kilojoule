#![feature(sized_type_properties)]
#![feature(alloc_layout_extra)]
#![feature(ptr_as_ref_unchecked)]

mod ast_node;
mod byte_vec;
mod evaluator;
mod heap_allocator;
mod json_lexer;
mod object_collector;
mod parser;
mod reader;
mod ring_buffer;
mod run_repl;
mod val;

// External exports
pub use byte_vec::{ByteVec, ByteVecTrait};
pub use evaluator::EvalCtx;
pub use heap_allocator::*;
pub use json_lexer::{JsonLexer, JsonLexerTrait, JsonToken};
pub use object_collector::ObjectCollector;
pub use reader::ReaderTrait;
pub use reader::*;
pub use ring_buffer::*;
pub use run_repl::run_repl;
