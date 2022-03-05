use std::path::Path;
use swc::config::util::BoolOrObject;
use swc::config::SourceMapsConfig;
use swc::{Compiler, IdentCollector};
use swc_common::collections::AHashMap;
use swc_common::comments::{Comments, SingleThreadedComments};
use swc_common::sync::Lrc;
use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
};
use swc_ecma_ast::{EsVersion, FnDecl};
use swc_ecma_codegen::Node;
use swc_ecma_parser::EsConfig;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_visit::{Visit, VisitWith};

pub struct CheckComments {
    comments: SingleThreadedComments,
    c: Compiler,
    pub result: Vec<String>,
}

pub fn check_no_comments_function(file_name: &str) -> CheckComments {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let fm = cm
        .load_file(Path::new(file_name))
        .expect("failed to load js file");
    let c = swc::Compiler::new(cm);

    let comments = SingleThreadedComments::default();
    let lexer = Lexer::new(
        // We want to parse ecmascript
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        // EsVersion defaults to es5
        Default::default(),
        StringInput::from(&*fm),
        Some(&comments),
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    let m = parser
        .parse_module()
        .map_err(|e| {
            // Unrecoverable fatal error occurred
            e.into_diagnostic(&handler).emit()
        })
        .expect("failed to parser module");

    // dbg!(&m);

    impl Visit for CheckComments {
        fn visit_fn_decl(&mut self, item: &FnDecl) {
            let has_comments = self.comments.get_leading(item.function.span.lo);
            if has_comments.is_none() {
                self.result.push(get_text(&self.c, &item.ident));
            }
        }
    }

    let mut check_comments = CheckComments {
        comments,
        c,
        result: Vec::new(),
    };

    m.visit_with(&mut check_comments);
    check_comments
}

pub fn get_text<T>(c: &Compiler, node: &T) -> String
where
    T: Node + VisitWith<IdentCollector>,
{
    c.print(
        node,
        None,
        None,
        false,
        EsVersion::Es2022,
        SourceMapsConfig::Bool(false),
        &AHashMap::default(),
        None,
        false,
        Some(BoolOrObject::Bool(true)),
    )
    .unwrap()
    .code
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use std::io::Write;

    use crate::utils;

    #[test]
    fn it_works() {
        let dir = env::temp_dir();
        assert_eq!(dir.display().to_string(), "/tmp");

        // Create a file inside of `std::env::temp_dir()`.
        let file_path = format!("{}/test.js", dir.display().to_string());
        let mut file = File::create(&file_path).unwrap();
        let js_source = "/**
        Comment for fct 1
        */
        function test(bc,b) {
            // comment1
            const a=1;
        }
        
        
        function test1(a,b) {
            //comment 2
            const a=1;
        }        
        ";
        writeln!(file, "{}", js_source).unwrap();
        let check_comments = utils::check_no_comments_function(file_path.as_str());
        assert_eq!(check_comments.result.len(), 1);
    }
}
